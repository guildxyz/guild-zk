### About

This repo contains experimental prototypes of ring signature algorithms that we
intend to use for zero-knowledge proofs.

### Notes

[0xPARC](https://0xparc.org/about) (Program for Applied Research in
Cryptography) is actively developing methods to prove public key derivation
from secret keys, verifying ECDSA signatures, etc. in zk snarks. They use
[circom](https://docs.circom.io/) for this which is a fairly low level language
for writing zk snark circuits. Problem is, that these circuits contain millions
of constraints and require a significant amount of computational resources
(tens of gigabytes of RAM).
[`StealthAirdrop`](https://github.com/nalinbhardwaj/stealthdrop) is a prototype
implementation making use of such circuits.

We are looking for alternative solutions (more lightweight) to prove that
someone has the corresponding private key for a given public key **without**
disclosing their public key. So far, [ring
signatures](https://en.wikipedia.org/wiki/Ring_signature) are quite promising,
as our usecase is exactly that we need to prove that we are part of a group of
addresses that hold some kind of tokens. Through
[Balancy](https://github.com/zgendao/balancy) we are able to construct such
address groups publicly. An entry-level (mathematical) introduction of how ring
signatures work can be found in [Monero's
handbook](https://www.getmonero.org/library/Zero-to-Monero-2-0-0.pdf). There is
a [Rust repo](https://github.com/edwinhere/nazgul) that implements the math in
this handbook which helped constructing our prototype.

There are some further practical issues to solve though. `StealthAirdrop` uses
[ECDSA verification circom scripts](https://github.com/0xPARC/circom-ecdsa)
that are way more bulkier than just proving secret/public key derivation
because there's no way to extract the bare private key from a wallet like
Metamask. You can read an interesting twitter thread about this
[here](https://twitter.com/0xPARC/status/1493704577002049537?s=20&t=X-5Bs1oWNjmbTASp2T82DA).

Ting signatures also make use of direct access to the secret key when
generating the proof. However, [there might exist a
solution](https://github.com/cloudflare/zkp-ecdsa) where EDCSA signature
verification happens via masking the public key via Pedersen commitments. The
whitepaper describing the algorithm can be found
[here](https://eprint.iacr.org/2021/1183.pdf), and it builds on [Groth's
seminal paper](https://eprint.iacr.org/2014/764.pdf). The advantage of this
method is that it doesn't seem to require direct access to the user's private
key. Thus, the user only signs a message (using Metamask for example) commits a
Pedersen commitment masking their public key and generates the proof based on
these values.

### Proposed flow

-   I. Balancy generates a list of eligible (Ethereum) addresses for a given guild.
-  II. Users whose address is found in this array (ring) may provide a ring signature
       that proves they are eligible to enter the guild.
- III. The guild backend receives the ring signature (proof) along with the ID of the guild
       and (Discord) ID of the user and verifies the proof.
