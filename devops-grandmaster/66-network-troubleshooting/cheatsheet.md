# Cheatsheet: Network Troubleshooting

## Essential Tools

| Tool | Purpose |
|------|---------|
| ping | Test connectivity |
| traceroute | Show path to host |
| mtr | Combined ping + traceroute |
| tcpdump | Capture packets |
| netstat/ss | Show connections |
| dig/nslookup | DNS lookup |
| curl/wget | HTTP requests |
| nmap | Port scanning |

## Common Commands
```bash
# Test connectivity
ping -c 3 example.com

# Trace route
traceroute example.com
mtr example.com

# Show connections
ss -tuln
ss -tp
netstat -tuln

# DNS lookup
dig example.com
dig +trace example.com
nslookup example.com

# HTTP debug
curl -v https://example.com
curl -I https://example.com  # Headers only
curl -o /dev/null -s -w "%{http_code} %{time_total}s\n" https://example.com

# Port scan
nmap -p 80,443 example.com

# Packet capture
tcpdump -i eth0 port 80
tcpdump -i eth0 host 10.0.0.1
tcpdump -i eth0 -w capture.pcap
```

## Kubernetes Network Debug
```bash
# Check pod networking
kubectl exec -it my-pod -- ping other-service
kubectl exec -it my-pod -- nslookup other-service
kubectl exec -it my-pod -- curl http://other-service:8080

# Check service endpoints
kubectl get endpoints my-service

# Check network policies
kubectl get networkpolicies

# Check DNS
kubectl exec -it my-pod -- cat /etc/resolv.conf
```

## Common Issues
```
Connection refused → Service not running
Connection timeout → Firewall blocking
DNS not resolving → CoreDNS issue
Slow response → Network latency or server overload
```
