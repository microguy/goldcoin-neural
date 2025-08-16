# GoldCoin Neural - Goldcoinpool.com Deployment Guide

## 🚀 Quick Start for Claude Code at Goldcoinpool.com

This guide will help Claude Code deploy and test the GoldCoin Neural AI wallet daemon on Goldcoinpool.com.

## Prerequisites

- Rust 1.75+ (for building from source)
- OR use pre-built binaries (recommended)
- Port 8122 available for RPC
- Existing pool infrastructure remains unchanged

## Installation Options

### Option 1: Pre-built Binary (Recommended)

```bash
# Download the GoldCoin Neural daemon
wget https://github.com/microguy/goldcoin-neural/releases/download/v0.1.0/goldcoin-neurald-linux-x64.tar.gz
tar -xzf goldcoin-neurald-linux-x64.tar.gz
cd goldcoin-neural

# Make executable
chmod +x goldcoin-neurald

# Test the binary
./goldcoin-neurald --version
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/microguy/goldcoin-neural.git
cd goldcoin-neural

# Build with Cargo
cargo build --release

# Binary will be at target/release/goldcoin-neurald
```

## Configuration

### 1. Create Configuration File

Create `goldcoin-neural.conf`:

```ini
# Network Configuration
network=mainnet
rpc_port=8122
rpc_user=goldcoinpool
rpc_password=<secure_password>

# Mining RPC Compatibility
mining_rpc_enabled=true
stratum_port=3333
getwork_enabled=true

# Pool-specific settings
pool_mode=true
pool_name=goldcoinpool.com

# GoldCoin Core Connection (if running alongside)
goldcoin_rpc_host=127.0.0.1
goldcoin_rpc_port=8121
goldcoin_rpc_user=<existing_user>
goldcoin_rpc_password=<existing_password>

# AI Features (optional for mining)
ai_enabled=true
neural_optimization=true
predictive_difficulty=true

# Security
defense_51_enabled=true
checkpoint_validation=true
golden_river_active=true

# Logging
log_level=info
log_file=/var/log/goldcoin-neural.log
```

### 2. Systemd Service (Recommended)

Create `/etc/systemd/system/goldcoin-neurald.service`:

```ini
[Unit]
Description=GoldCoin Neural AI Daemon
After=network.target

[Service]
Type=simple
User=goldcoin
Group=goldcoin
WorkingDirectory=/opt/goldcoin-neural
ExecStart=/opt/goldcoin-neural/goldcoin-neurald --config /etc/goldcoin-neural.conf
Restart=always
RestartSec=10

# Security
PrivateTmp=true
NoNewPrivileges=true
ProtectSystem=strict
ReadWritePaths=/var/lib/goldcoin-neural /var/log/goldcoin-neural

[Install]
WantedBy=multi-user.target
```

## Pool Integration

### 1. Update Pool Configuration

Modify your pool's configuration to point to the Neural daemon:

#### For NOMP-based pools:

```json
{
  "coin": {
    "name": "Goldcoin",
    "symbol": "GLD",
    "algorithm": "scrypt"
  },
  "daemon": {
    "host": "127.0.0.1",
    "port": 8122,  // Neural daemon RPC port
    "user": "goldcoinpool",
    "password": "<password>"
  }
}
```

#### For MPOS-based pools:

```php
$config['wallet']['host'] = '127.0.0.1';
$config['wallet']['port'] = 8122;  // Neural daemon RPC port
$config['wallet']['username'] = 'goldcoinpool';
$config['wallet']['password'] = '<password>';
```

### 2. Testing Mining Compatibility

Run these tests to verify the pool works correctly:

```bash
# Test basic RPC connectivity
curl -u goldcoinpool:<password> -X POST http://localhost:8122 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"getmininginfo","params":[],"id":1}'

# Test getblocktemplate
curl -u goldcoinpool:<password> -X POST http://localhost:8122 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"getblocktemplate","params":[{"capabilities":["coinbasetxn","workid","coinbase/append"]}],"id":1}'

# Test getwork (for legacy miners)
curl -u goldcoinpool:<password> -X POST http://localhost:8122 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"getwork","params":[],"id":1}'
```

## Monitoring

### 1. Health Check Endpoint

The Neural daemon provides a health check endpoint:

```bash
curl http://localhost:8122/health
```

Expected response:
```json
{
  "status": "healthy",
  "block_height": 850000,
  "difficulty": 1234567.89,
  "golden_river_active": true,
  "defense_51_status": "monitoring",
  "connections": 8,
  "mining_enabled": true
}
```

### 2. Pool-Specific Metrics

Monitor these metrics for pool operation:

```bash
# Get pool statistics
curl -u goldcoinpool:<password> -X POST http://localhost:8122 \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","method":"getpoolstats","params":[],"id":1}'
```

Response includes:
- Current hashrate
- Active miners
- Pending blocks
- Recent blocks found
- Fee predictions (AI-powered)

### 3. Logging

Monitor logs for issues:

```bash
# Follow the logs
tail -f /var/log/goldcoin-neural.log

# Check for mining-specific events
grep "mining" /var/log/goldcoin-neural.log

# Monitor 51% defense
grep "defense_51" /var/log/goldcoin-neural.log
```

## Testing Checklist

### Phase 1: Basic Compatibility ✅

- [ ] Daemon starts successfully
- [ ] RPC port 8122 is accessible
- [ ] Authentication works
- [ ] getmininginfo returns valid data
- [ ] getblocktemplate returns valid template
- [ ] getwork returns valid work

### Phase 2: Pool Integration 🔄

- [ ] Pool connects to Neural daemon
- [ ] Pool receives block templates
- [ ] Miners can connect to pool
- [ ] Shares are accepted
- [ ] Block submission works
- [ ] Payouts function correctly

### Phase 3: AI Features Testing 🤖

- [ ] Predictive difficulty adjustments
- [ ] Fee optimization working
- [ ] 51% defense monitoring active
- [ ] Golden River algorithm functioning
- [ ] Checkpoint validation working

### Phase 4: Performance Testing 📊

- [ ] Latency comparable to original daemon
- [ ] Can handle pool's miner load
- [ ] Memory usage stable
- [ ] CPU usage reasonable
- [ ] No memory leaks after 24h

## Rollback Plan

If issues occur, you can instantly rollback:

```bash
# Stop Neural daemon
systemctl stop goldcoin-neurald

# Point pool back to original daemon
# Update pool config to use port 8121 (original)

# Restart pool
systemctl restart pool-software
```

## Support Commands

### View current mining status:
```bash
./goldcoin-neurald mining-status
```

### Test miner connection:
```bash
./goldcoin-neurald test-miner --address <miner_address>
```

### Benchmark RPC performance:
```bash
./goldcoin-neurald benchmark-rpc --requests 1000
```

## Expected Behavior

### What Should Work Immediately:
✅ All existing miners continue working unchanged
✅ Pool software needs only port change (8121 → 8122)
✅ Block finding and submission
✅ Share validation
✅ Difficulty adjustments
✅ All standard RPC calls

### New Features Available:
🆕 AI-powered fee predictions
🆕 Natural language transaction queries (via separate API)
🆕 Predictive network analysis
🆕 Enhanced security monitoring
🆕 Real-time 51% attack detection

## Troubleshooting

### Issue: Pool can't connect
```bash
# Check if daemon is running
ps aux | grep goldcoin-neurald

# Check if port is listening
netstat -tlnp | grep 8122

# Test RPC directly
curl -u user:pass http://localhost:8122 -d '{"method":"getinfo"}'
```

### Issue: Miners rejected
```bash
# Check daemon logs
tail -100 /var/log/goldcoin-neural.log | grep ERROR

# Verify scrypt algorithm
./goldcoin-neurald check-algorithm
```

### Issue: High CPU usage
```bash
# Disable AI features for pure mining
echo "ai_enabled=false" >> goldcoin-neural.conf
systemctl restart goldcoin-neurald
```

## Contact

For pool-specific support:
- Create issue: https://github.com/microguy/goldcoin-neural/issues
- Tag: @microguy
- Mention: "Goldcoinpool.com testing"

---

## Quick Test Script

Save this as `test-pool-integration.sh`:

```bash
#!/bin/bash

echo "Testing GoldCoin Neural Pool Integration"
echo "========================================="

# Configuration
RPC_USER="goldcoinpool"
RPC_PASS="your_password"
RPC_URL="http://localhost:8122"

# Test 1: Basic connectivity
echo -n "Testing RPC connectivity... "
if curl -s -u $RPC_USER:$RPC_PASS $RPC_URL -d '{"method":"getblockcount"}' > /dev/null; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
    exit 1
fi

# Test 2: Mining RPC
echo -n "Testing getmininginfo... "
if curl -s -u $RPC_USER:$RPC_PASS $RPC_URL -d '{"method":"getmininginfo"}' | grep -q "difficulty"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

# Test 3: Block template
echo -n "Testing getblocktemplate... "
if curl -s -u $RPC_USER:$RPC_PASS $RPC_URL -d '{"method":"getblocktemplate","params":[{"capabilities":["coinbasetxn"]}]}' | grep -q "height"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

# Test 4: Getwork (legacy)
echo -n "Testing getwork... "
if curl -s -u $RPC_USER:$RPC_PASS $RPC_URL -d '{"method":"getwork"}' | grep -q "target"; then
    echo "✅ PASS"
else
    echo "❌ FAIL"
fi

echo ""
echo "Pool integration test complete!"
```

---

**Ready to Deploy!** 🚀

The GoldCoin Neural daemon is designed to be a drop-in replacement for mining operations. Just point your pool to port 8122 instead of 8121, and all your miners will continue working exactly as before, with AI enhancements running in the background!