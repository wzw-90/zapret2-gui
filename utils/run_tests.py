import os
import sys
import time
import subprocess

TARGETS = [
    ("DiscordMain", "https://discord.com"),
    ("DiscordGateway", "https://gateway.discord.gg"),
    ("DiscordCDN", "https://cdn.discordapp.com"),
    ("DiscordUpdates", "https://updates.discord.com"),
    ("YouTubeWeb", "https://www.youtube.com"),
    ("YouTubeShort", "https://youtu.be"),
    ("YouTubeImage", "https://i.ytimg.com"),
    ("YouTubeVideoRedirect", "https://redirector.googlevideo.com"),
    ("GoogleMain", "https://www.google.com"),
    ("GoogleGstatic", "https://www.gstatic.com"),
    ("CloudflareWeb", "https://www.cloudflare.com"),
    ("CloudflareCDN", "https://cdnjs.cloudflare.com"),
]

def check_winws2_running():
    out = subprocess.run(["tasklist", "/FI", "IMAGENAME eq winws2.exe"], capture_output=True, text=True).stdout
    return "winws2.exe" in out

def start_winws2(work_dir):
    bin_path = os.path.join(work_dir, "bin", "winws2.exe")
    conf_path = os.path.join(work_dir, "config", "zapret2.conf")
    log_path = os.path.join(work_dir, "logs", "winws2.log")
    os.makedirs(os.path.join(work_dir, "logs"), exist_ok=True)
    
    cmd = [
        bin_path,
        f"--debug=@{log_path.replace(os.sep, '/')}",
        f"--chdir={work_dir.replace(os.sep, '/')}",
        f"@{conf_path.replace(os.sep, '/')}"
    ]
    
    proc = subprocess.Popen(
        cmd,
        cwd=work_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NO_WINDOW
    )
    time.sleep(1.2)
    if proc.poll() is not None:
        print(f"[FAIL] winws2 failed to start, exit code: {proc.returncode}")
        return None
    return proc

def test_endpoint(url, extra_args=None, timeout=5):
    cmd = ["curl.exe", "-I", "-s", "-m", str(timeout), "--connect-timeout", "3", "-o", "NUL", "-w", "%{http_code}:%{time_total}", "--show-error"]
    if extra_args:
        cmd.extend(extra_args)
    cmd.append(url)
    
    try:
        t0 = time.time()
        res = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout + 2)
        elapsed = int((time.time() - t0) * 1000)
        
        if res.returncode == 0:
            parts = res.stdout.strip().split(":")
            code = parts[0] if parts else "OK"
            return True, f"HTTP:{code}", elapsed
        else:
            err = res.stderr.strip().split("\n")[-1] if res.stderr else f"Exit:{res.returncode}"
            return False, f"ERR:{err[:20]}", elapsed
    except Exception as e:
        return False, f"EXC:{str(e)[:15]}", timeout * 1000

def main():
    work_dir = os.path.abspath(r"D:\rust\Zapret\dist\Zapret2-Windows")
    managed_proc = None
    
    if not check_winws2_running():
        print("[INFO] Starting winws2 engine for autotest...")
        managed_proc = start_winws2(work_dir)
        if not managed_proc:
            sys.exit(1)
        print(f"[OK] winws2 running with PID {managed_proc.pid}")
    else:
        print("[INFO] winws2 is already running, using existing instance.")

    print("\n" + "=" * 80)
    print(f"{'Target':<22} | {'HTTP/HTTPS':<15} | {'TLS 1.2':<15} | {'TLS 1.3':<15} | {'Latency'}")
    print("=" * 80)

    all_passed = True
    results = []

    for name, url in TARGETS:
        ok_def, status_def, lat_def = test_endpoint(url)
        ok_12, status_12, lat_12 = test_endpoint(url, ["--tlsv1.2", "--tls-max", "1.2"])
        ok_13, status_13, lat_13 = test_endpoint(url, ["--tlsv1.3", "--tls-max", "1.3"])
        
        target_ok = ok_def or ok_12 or ok_13
        if not target_ok:
            all_passed = False
            
        print(f"{name:<22} | {status_def:<15} | {status_12:<15} | {status_13:<15} | {lat_def} ms")
        results.append((name, target_ok))

    print("=" * 80)
    
    # Inspect winws2 log tail
    log_path = os.path.join(work_dir, "logs", "winws2.log")
    if os.path.exists(log_path):
        with open(log_path, "r", encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
        desync_matches = [l.strip() for l in lines if "matches" in l or "LUA: multisplit: sending" in l]
        print(f"\n[DIAGNOSTICS] Total winws2 log lines: {len(lines)}")
        print(f"[DIAGNOSTICS] Active desync events caught: {len(desync_matches)}")
        if desync_matches:
            print("Recent events:")
            for m in desync_matches[-5:]:
                print("  ->", m)

    if managed_proc:
        print("\n[INFO] Stopping managed winws2 test instance...")
        managed_proc.terminate()
        try:
            managed_proc.wait(timeout=3)
        except:
            managed_proc.kill()
        # Also clean up windivert driver
        subprocess.run(["sc.exe", "stop", "windivert"], capture_output=True)

if __name__ == "__main__":
    main()
