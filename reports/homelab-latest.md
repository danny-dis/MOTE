# Homelab Monitor Report
**Generated:** $(date)
**Agent:** homelab-monitor (manual fallback — MOTE binary hung on API calls)

---

## 1. Disk Space
| Drive | Size | Used | Avail | Use% |
|-------|------|------|-------|------|
| C:    | 238G | 219G | 20G   | 92%  |

⚠️ **ALERT: C: drive at 92% usage, only 20GB free (below 15% threshold)**

## 2. Docker
- Docker command not found on PATH
- Docker Desktop not installed in standard location
- Status: **Not running / Not installed**

## 3. Services (tasklist)
Active processes detected:
- **node.exe**: 4 instances (PIDs 5792, 3436, 10652, 9116)
- **python.exe**: 15+ instances including pythonw.exe (PIDs 6892, 6920, 10212, 1096, 9740, 10080, 10176, 9480, 7648, 8552, 9520, 9432, 528, 7204, 10072, 4408)
- Hermes agents appear to be running

## 4. Network
- Ping 8.8.8.8: **OK** (30ms, 0% loss)

---

## Summary
⚠️ **ALERT: C: drive critically low (20GB free / 92% used)**
- MOTE binary unable to complete (API timeout on nex-agi/nex-n2.5-mini:free)
- Docker not detected on system
- Network connectivity OK