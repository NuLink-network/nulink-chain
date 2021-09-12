---
sort: 1
---

# Proxy Re-encryption (PRE)
***
Proxy re-encryption (PRE) is a type of public-key encryption (PKE) that allows a proxy entity to transform or re-encrypt data from one public key to another, without having access to the underlying plaintext or private keys.
The operation process of proxy re-encryption is as follows: 
1. The publisher A encrypts the data M with its own public key into C1, then A sends the ciphertext C1 to the proxy and generates a re-encryption key for him, which is calculated by A self. 
2. Next, the proxy uses the re-encryption key to convert the ciphertext C1 into a ciphertext C2 which can be decrypted by B with his secret key, and sends it to B. The agent only provides computing conversion services and cannot get plaintext. 
3. B decrypts the plaintext M that A wants to share in secret. 

Proxy re-encryption plays a role in data transmission and protect privacy in NuLink.