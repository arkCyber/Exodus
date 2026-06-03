# Exodus Microservices

This directory contains standalone microservice binaries for the Exodus browser architecture.

## Service Structure

Each microservice is a standalone Rust binary that:
- Communicates via Unix Domain Sockets (UDS)
- Uses JSON-RPC 2.0 protocol
- Runs independently of the main browser process
- Can be started/stopped/restarted independently

## Services

### exodus-rag-service
Independent RAG (Retrieval-Augmented Generation) service with:
- Local vector storage using sled/qdrant
- File system watching for automatic re-indexing
- Incremental embedding updates
- Semantic search API

### exodus-crypto-service
Web3 blockchain service with:
- EVM transaction simulation
- Security auditing
- Wallet integration
- Multi-chain support

### exodus-os-service
OS automation service with:
- Keyboard/mouse simulation
- Window management
- Process control
- System-level automation

### exodus-depin-node
Distributed compute node with:
- P2P networking (libp2p)
- Compute resource management
- DePIN protocol integration

## Running Services

```bash
# Build all services
cargo build --release

# Run individual service
./target/release/exodus-rag-service

# Run with custom socket path
./target/release/exodus-rag-service --socket /tmp/exodus-rag.sock
```

## Communication Protocol

All services use JSON-RPC 2.0 over Unix Domain Sockets:

```json
{
  "jsonrpc": "2.0",
  "method": "service.method",
  "params": {...},
  "id": 1
}
```

## Service Management

Services are managed by the main Exodus browser through the microservice bus:
- Automatic registration on startup
- Health monitoring
- Auto-restart on failure
- Graceful shutdown
