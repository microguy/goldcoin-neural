#!/bin/bash

# Safe Docker Testing Script for Goldcoinpool.com
# This script sets up an isolated test environment

set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║     GoldCoin Neural - Docker Test for Goldcoinpool.com    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
POOL_RPC_USER="${POOL_RPC_USER:-goldcoinpool}"
POOL_RPC_PASS="${POOL_RPC_PASS:-testpass123}"
PRODUCTION_PORT=8121
TEST_PORT=8122

echo -e "${YELLOW}🔍 Checking environment...${NC}"

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not installed${NC}"
    echo "Install with: curl -fsSL https://get.docker.com | sh"
    exit 1
fi

# Check if production goldcoind is running
if lsof -Pi :$PRODUCTION_PORT -sTCP:LISTEN -t >/dev/null ; then
    echo -e "${GREEN}✅ Production goldcoind detected on port $PRODUCTION_PORT${NC}"
    echo -e "${YELLOW}   Neural will run on port $TEST_PORT (isolated)${NC}"
else
    echo -e "${YELLOW}⚠️  No production goldcoind detected${NC}"
fi

# Create test configuration
echo -e "\n${YELLOW}📝 Creating test configuration...${NC}"
cat > goldcoin-neural.conf <<EOF
# GoldCoin Neural Test Configuration
# For Goldcoinpool.com Testing

network=mainnet
rpc_port=$TEST_PORT
rpc_user=$POOL_RPC_USER
rpc_password=$POOL_RPC_PASS

# Mining
mining_rpc_enabled=true
pool_mode=true
pool_name=goldcoinpool.com

# Connect to production node (optional)
goldcoin_rpc_host=host.docker.internal
goldcoin_rpc_port=$PRODUCTION_PORT

# AI Features (can be disabled for pure mining test)
ai_enabled=true
neural_optimization=true
predictive_difficulty=true

# Security
defense_51_enabled=true
checkpoint_validation=true
golden_river_active=true

# Logging
log_level=info
EOF

echo -e "${GREEN}✅ Configuration created${NC}"

# Build Docker image
echo -e "\n${YELLOW}🔨 Building Docker image...${NC}"
echo "This will take 5-10 minutes on first run..."

if docker build -t goldcoin-neural:test .; then
    echo -e "${GREEN}✅ Docker image built successfully${NC}"
else
    echo -e "${RED}❌ Docker build failed${NC}"
    exit 1
fi

# Run container
echo -e "\n${YELLOW}🚀 Starting test container...${NC}"

# Stop any existing test container
docker stop goldcoin-neural-test 2>/dev/null || true
docker rm goldcoin-neural-test 2>/dev/null || true

# Run with resource limits
docker run -d \
    --name goldcoin-neural-test \
    --memory="512m" \
    --cpus="0.5" \
    -p $TEST_PORT:$TEST_PORT \
    -v $(pwd)/goldcoin-neural.conf:/home/goldcoin/.goldcoin-neural.conf:ro \
    --restart unless-stopped \
    goldcoin-neural:test

# Wait for startup
echo -e "${YELLOW}⏳ Waiting for daemon to start...${NC}"
sleep 5

# Test RPC connection
echo -e "\n${YELLOW}🔍 Testing RPC connection...${NC}"

test_rpc() {
    curl -s -u $POOL_RPC_USER:$POOL_RPC_PASS \
        -X POST http://localhost:$TEST_PORT \
        -H 'Content-Type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$1\",\"params\":[],\"id\":1}" 2>/dev/null
}

# Test getblockcount
if test_rpc "getblockcount" | grep -q "result"; then
    echo -e "${GREEN}✅ RPC connection successful${NC}"
else
    echo -e "${RED}❌ RPC connection failed${NC}"
    echo "Check logs: docker logs goldcoin-neural-test"
    exit 1
fi

# Test mining RPCs
echo -e "\n${YELLOW}🔍 Testing mining RPCs...${NC}"

if test_rpc "getmininginfo" | grep -q "difficulty"; then
    echo -e "${GREEN}✅ getmininginfo works${NC}"
fi

if test_rpc "getblocktemplate" | grep -q "height"; then
    echo -e "${GREEN}✅ getblocktemplate works${NC}"
fi

# Show status
echo -e "\n${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✅ GoldCoin Neural is running in Docker${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo "Container name: goldcoin-neural-test"
echo "RPC endpoint: http://localhost:$TEST_PORT"
echo "Credentials: $POOL_RPC_USER / [configured password]"
echo ""
echo -e "${YELLOW}📋 Next steps:${NC}"
echo "1. Point a test pool instance to port $TEST_PORT"
echo "2. Monitor: docker logs -f goldcoin-neural-test"
echo "3. Stats: docker stats goldcoin-neural-test"
echo "4. Stop: docker stop goldcoin-neural-test"
echo "5. Remove: docker rm goldcoin-neural-test"
echo ""
echo -e "${GREEN}Production on port $PRODUCTION_PORT is unaffected!${NC}"