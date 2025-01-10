# hyperchain-da
Common clients and contracts for DA solutions for ZK chains.

> [!WARNING]  
> This repository has been archived. Development has been moved to the [zksync-era](https://github.com/matter-labs/zksync-era/tree/main/core/node/da_clients) repository. Please use it instead.

# Clients
The clients from this repository are going to be imported by the [zksync-era](https://github.com/matter-labs/zksync-era) and used by the DataAvailabilityDispatcher component.

- It is assumed that the DA client is only serving as a connector between the ZK chain's sequencer and the DA layer. 
- The DA client is not supposed to be a standalone application, but rather a library that is used by the sequencer.
- The logic of the retries is implemented in the sequencer, not in the DA clients.
- The `get_inclusion_data` has to return the data only when the state roots are relayed to the L1 verification contract.

---
The examples of the clients can be found [here](https://github.com/matter-labs/zksync-era/tree/feat-validium-with-da/core/lib/da_client/src/clients).

If you want to add a new client - you need to add your implementation of the `DataAvailabilityInterface` trait to the `src/clients` directory.

# Contracts
The contracts consist of two parts:
- L2 contract
  - Produces the DA commitment, which is then relayed to the L1 via system logs.
- L1 contract
  - Decodes the inclusion data provided by the sequencer.
  - Matches the DA commitment from logs with the inclusion data.
  - Calls the verification contract (Attestation Bridge/Blobstream...) with the inclusion data.

