import os
import glob
import re

bcinr_files = glob.glob("/Users/sac/chatmangpt/bcinr/crates/bcinr-logic/src/algorithms/*.rs")
bcinr_names = [os.path.basename(f).replace('.rs', '') for f in bcinr_files]

algorithms_300 = [
    "Chase-Lev Work-Stealing Deque", "LMAX Disruptor Pattern", "MPSC Ring Buffer", "SPMC Ring Buffer", 
    "MPMC Bounded Queue", "SPSC Bipuffer / Bounded Ring", "Stackful Coroutines / Fibers", "Stackless Coroutines", 
    "Protothreads", "Seqlocks", "Ticket Locks", "MCS Locks", "CLH Locks", "Flat Combining", 
    "Elimination Backoff Stack", "Hazard Pointers", "Epoch-Based Reclamation (EBR)", "Quiescent State-Based Reclamation (QSBR)", 
    "Read-Copy-Update (RCU)", "Reactor Pattern", "Proactor Pattern", "epoll / kqueue Integration", "io_uring Integration", 
    "Hardware Transactional Memory (Intel TSX)", "Software Transactional Memory (STM)", "Concurrent Skip List", 
    "Concurrent Hash Trie (Ctrie)", "Treiber Stack", "Michael-Scott Queue", "Left-Right Pattern", 
    "Thread-per-core (Pinned) architecture", "False Sharing Padder / Cache-line alignment", "Backoff/Yield strategies", 
    "Active Message passing (Actor Model)", "Wait-free Consensus Protocols",
    "Bump Pointer Arena Allocator", "Slab Allocator", "Buddy Memory Allocator", "Free List Allocator", 
    "Hoard Allocator Algorithm", "Thread-Local Caches (TCMalloc style)", "Segregated Fits Allocator", 
    "Object Pools (Lock-Free)", "Memory-Mapped Files (mmap)", "Virtual Memory Area (VMA) paging control", 
    "madvise / Huge Pages", "Cache-Oblivious B-Trees", "Cache-Oblivious Matrix Transposition", "Z-order Curve Mapping", 
    "Morton Codes", "Hilbert Curve Mapping", "Struct of Arrays (SoA)", "Array of Structs of Arrays (AoSoA)", 
    "Pointer Tagging / NaN-boxing", "Generational Arenas", "Slot Maps", "Bitmapped Block Allocation", 
    "Page-aligned allocation algorithms", "NUMA-aware allocation (First-touch policy)", "Garbage Collection (Mark & Sweep)", 
    "Garbage Collection (Cheney's Algorithm)", "Reference Counting (Arc/Rc) with Cycle Detection", "Fat Pointers", 
    "Inline dynamically-sized types (DSTs)", "Small String Optimization (SSO)",
    "Hopcroft-Karp Algorithm", "Tarjan's Strongly Connected Components (SCC)", "Kosaraju's Algorithm", 
    "Alpha Miner Algorithm", "Heuristic Miner Algorithm", "Fuzzy Miner Algorithm", "Inductive Miner - Directly Follows (IMdf)", 
    "Inductive Miner - Incompleteness (IMin)", "Split Miner Algorithm", "Region-Based Mining (State-based)", 
    "Region-Based Mining (Language-based)", "Ullmann's Subgraph Isomorphism Algorithm", "VF2 Subgraph Isomorphism", 
    "Dijkstra's Algorithm", "A* Search Algorithm", "IDA* (Iterative Deepening A*)", "K-Shortest Paths (Yen's Algorithm)", 
    "Floyd-Warshall Algorithm", "Johnson's Algorithm", "Bellman-Ford Algorithm", "Kahn's Algorithm", 
    "Hierarchical Clustering (Agglomerative)", "K-Means Clustering", "DBSCAN (Density-Based Spatial Clustering)", 
    "OPTICS Algorithm", "PageRank", "HITS Algorithm", "Ford-Fulkerson Algorithm", "Edmonds-Karp Algorithm", 
    "Dinic's Algorithm", "Push-Relabel Algorithm", "Karger's Algorithm", "Bipartite Graph Testing", 
    "Eulerian Path Detection", "Hamiltonian Path Detection", "Minimum Spanning Tree (Kruskal's)", 
    "Minimum Spanning Tree (Prim's)", "Bron-Kerbosch Algorithm", "Graph Coloring (Welsh-Powell)", 
    "Biconnected Components (Hopcroft-Tarjan)", "Dominator Tree Algorithm", "Post-Dominator Analysis", 
    "Control Dependence Graph construction", "Program Dependence Graph construction", "Data Flow Graph Analysis", 
    "LCA (Lowest Common Ancestor)", "Heavy-Light Decomposition", "Centroid Decomposition", "Tree Isomorphism", 
    "Siphon and Trap Computation",
    "SIMD JSON Parsing (simdjson style)", "AVX-512 Mask Registers", "SIMD Bitonic Sort", "SIMD Bit-Reversal Permutation", 
    "SIMD Prefix Sum (Scan)", "Vectorized String Matching (Hyperscan/Teddy)", "SIMD Hash Table Probing", 
    "SIMD Base64 Encoding/Decoding", "SIMD UTF-8 Validation", "Hardware CRC32 Instructions", "AES-NI Instructions", 
    "Popcnt Intrinsic", "Bit Manipulation Instruction Set (BMI1/BMI2)", "Parallel Bit Extract (PEXT)", 
    "Parallel Bit Deposit (PDEP)", "Vectorized Matrix Multiplication", "SIMD Floating Point Clamp/Max/Min", 
    "FMA (Fused Multiply-Add)", "Software Pipelining", "Loop Unrolling (Duff's Device equivalents)", 
    "Prefetching Algorithms", "Gather/Scatter Instructions (AVX-512)", "Branchless Binary Search", "SIMD Prefix XOR", 
    "Vectorized Bloom Filter Probing",
    "HyperLogLog (HLL)", "HyperLogLog++", "Bloom Filter", "Counting Bloom Filter", "Cuckoo Filter", 
    "Quotient Filter", "Ribbon Filter", "Xor Filter", "Space-Saving Algorithm", "Misra-Gries Summary", 
    "Lossy Counting Algorithm", "Sticky Sampling Algorithm", "Count-Sketch", "AMS Sketch (Alon-Matias-Szegedy)", 
    "T-Digest", "HDR Histogram", "Q-Digest", "K-Minimum Values (KMV)", "MinHash", "SimHash", 
    "Reservoir Sampling", "Sliding Window Reservoir Sampling (Algorithm L/Z)", "Exponential Moving Average (EMA)", 
    "Welford's Online Algorithm", "Kalman Filter", "Particle Filter", "Gk-Quantiles (Greenwald-Khanna)", 
    "Cormode-Muthukrishnan Algorithm", "LogLog Sketch", "Distributed Top-K Algorithm",
    "Aho-Corasick Algorithm", "Knuth-Morris-Pratt (KMP)", "Boyer-Moore String Search", "Rabin-Karp Algorithm", 
    "Z Algorithm", "Suffix Array Construction (SA-IS)", "Suffix Tree Construction (Ukkonen's Algorithm)", 
    "Suffix Automaton (DAWG)", "Burrows-Wheeler Transform (BWT)", "FM-Index", "Lempel-Ziv 77 (LZ77)", 
    "LZ78", "LZW (Lempel-Ziv-Welch)", "Huffman Coding", "Arithmetic Coding", "Run-Length Encoding (RLE)", 
    "Delta Encoding", "Elias Gamma / Delta Coding", "Golomb Coding", "Varint / LEB128 Encoding", 
    "ZigZag Encoding", "Snappy Compression Algorithm", "Zstandard (Zstd) Algorithm", "Brotli Compression Algorithm", 
    "Roaring Bitmaps", "EWAH Bitmaps", "WAH (Word Aligned Hybrid) Compression", "Concise (Compressed 'n' Composable Integer Set)", 
    "Trie (Prefix Tree)", "Patricia Trie / Radix Tree",
    "MurmurHash3", "CityHash", "xxHash", "FarmHash", "SipHash", "HighwayHash", "FKS Perfect Hashing", 
    "Cuckoo Hashing", "Hopscotch Hashing", "Robin Hood Hashing", "SwissTable (Google's Abseil)", 
    "Linear Probing with Robin Hood heuristics", "Consistent Hashing (Ketama)", "Rendezvous Hashing", 
    "Jump Consistent Hashing", "Locality-Sensitive Hashing (LSH)", "Pearson Hashing", "Merkle Trees", 
    "Merkle-Damgård Construction", "Consistent Overhead Byte Stuffing (COBS)", "Judy Arrays", "B-Tree / B+ Tree", 
    "LSM-Tree (Log-Structured Merge-Tree)", "Fractal Tree Index", "Quadtree", "K-D Tree", "R-Tree", 
    "Vantage-Point Tree (VP-Tree)", "BK-Tree (Burkhard-Keller)", "Ternary Search Tree",
    "Paxos Consensus Algorithm", "Raft Consensus Algorithm", "Zab (ZooKeeper Atomic Broadcast)", 
    "Gossip Protocol / Epidemic Broadcast", "Vector Clocks", "Lamport Timestamps", "Matrix Clocks", 
    "TrueTime (Spanner)", "Snowflake IDs / Twitter Snowflake", "PN-Counter (Positive-Negative CRDT)", 
    "G-Counter (Grow-only CRDT)", "LWW-Element-Set (Last-Writer-Wins CRDT)", "OR-Set (Observed-Remove CRDT)", 
    "Sequence CRDTs (Logoot / Treedoc)", "Merkle CRDTs", "Two-Phase Commit (2PC)", "Three-Phase Commit (3PC)", 
    "Saga Pattern (Choreography/Orchestration)", "Chord Distributed Hash Table Algorithm", "Kademlia DHT Algorithm", 
    "Pastry DHT Algorithm", "Bully Algorithm", "Ring Election Algorithm", "Chandy-Lamport Algorithm", 
    "Byzantine Fault Tolerance (PBFT)",
    "Epsilon-Greedy Algorithm", "Thompson Sampling", "Upper Confidence Bound (UCB1, UCB-V)", "EXP3 Algorithm", 
    "Q-Learning (Tabular)", "Double Q-Learning", "SARSA (State-Action-Reward-State-Action)", "Expected SARSA", 
    "Deep Q-Networks (DQN)", "Proximal Policy Optimization (PPO)", "Trust Region Policy Optimization (TRPO)", 
    "Actor-Critic Methods (A2C / A3C)", "Soft Actor-Critic (SAC)", "Monte Carlo Tree Search (MCTS)", 
    "Ant Colony Optimization (ACO)", "Particle Swarm Optimization (PSO)", "Simulated Annealing", 
    "Genetic Algorithms (GA)", "Covariance Matrix Adaptation Evolution Strategy (CMA-ES)", "Gradient Descent / Stochastic Gradient Descent (SGD)", 
    "Adam Optimizer", "L-BFGS (Limited-memory Broyden–Fletcher–Goldfarb–Shanno)", "Simplex Algorithm", 
    "Branch and Bound", "Strassen Algorithm", "Cooley-Tukey FFT Algorithm", "Karatsuba Algorithm", 
    "Montgomery Multiplication", "Newton-Raphson Method", "Fast Inverse Square Root (`0x5f3759df`)",
    "AES-GCM (Galois/Counter Mode)", "ChaCha20-Poly1305", "SHA-256 / SHA-3", "BLAKE3", "HMAC (Hash-based Message Authentication Code)", 
    "RSA / ECC (Elliptic Curve Cryptography)", "Ed25519", "Diffie-Hellman Key Exchange", "Argon2 / scrypt / bcrypt", 
    "Shamir's Secret Sharing", "Zero-Knowledge Proofs (zk-SNARKs)", "Bulletproofs", "Homomorphic Encryption (Fully/Partially)", 
    "Ring Signatures", "Oblivious RAM (ORAM)"
]

def normalize(s):
    s = s.lower()
    s = re.sub(r'[^a-z0-9]', '', s)
    return s

norm_bcinr = [normalize(n) for n in bcinr_names]

missing = []
for algo in algorithms_300:
    norm_algo = normalize(algo)
    
    # Check if words in algo are in bcinr
    found = False
    for n in norm_bcinr:
        if norm_algo in n or n in norm_algo:
            found = True
            break
            
    if not found:
        # try word match
        words = set(re.findall(r'[a-z0-9]+', algo.lower()))
        for n in bcinr_names:
            n_words = set(re.findall(r'[a-z0-9]+', n.lower()))
            if len(words.intersection(n_words)) >= 2: # At least two words match
                found = True
                break
                
    if not found:
        missing.append(algo)

print(f"Total algos: {len(algorithms_300)}")
print(f"Missing algos: {len(missing)}")

import json
with open('missing.json', 'w') as f:
    json.dump(missing, f)

# Let's generate a file for each missing algorithm
os.makedirs("src/bcinr_extended", exist_ok=True)

mod_rs = []
for i, algo in enumerate(missing):
    safe_name = re.sub(r'[^a-zA-Z0-9]', '_', algo.lower())
    safe_name = re.sub(r'_+', '_', safe_name).strip('_')
    
    with open(f"src/bcinr_extended/{safe_name}.rs", 'w') as f:
        f.write(f"//! Implementation of {algo}\n")
        f.write(f"pub fn {safe_name}_stub() {{ unimplemented!() }}\n")
        
    mod_rs.append(f"pub mod {safe_name};")

with open("src/bcinr_extended/mod.rs", 'w') as f:
    f.write("\n".join(mod_rs))

