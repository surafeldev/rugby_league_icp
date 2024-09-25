# Rugby League Player Transfer Management System

## Overview

The Rugby League Player Transfer Management System is a decentralized application designed to manage rugby league player profiles, transfers, and transfer offers. Built using Rust, the application leverages the Internet Computer (IC) for decentralized and reliable data storage and access.

## Features

- **Player Management**: Create and retrieve player profiles.
- **Transfer Management**: Create and track player transfers between teams.
- **Transfer Offers**: Create, accept, and reject transfer offers.
- **Data Storage**: Persistent storage for player profiles, transfers, and offers.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (version 1.60 or later)
- [IC SDK](https://sdk.dfinity.org/docs/developers-guide/install-upgrade.html)

### Installation

1. **Clone the Repository**

   ```bash
   git clone https://github.com/your-repo/rugby-league-player-transfer-management.git
   cd rugby-league-player-transfer-management
   ```

2. **Install Dependencies**

   Ensure you have the necessary Rust dependencies installed by running:

   ```bash
   cargo build
   ```

3. **Deploy to Internet Computer**

   Follow the [IC SDK deployment guide](https://sdk.dfinity.org/docs/developers-guide/deploy.html) to deploy the project to the Internet Computer.

## Usage

### Player Management

- **Create a Player**

  Use the `create_player` update call to add a new player. Provide the player details in the `PlayerPayload`.

  ```json
  {
    "name": "John Doe",
    "position": "Fly-half",
    "current_team": "Warriors",
    "market_value": 1000000,
    "contract_until": 2025,
    "age": 28,
    "nationality": "Australian"
  }
  ```

- **Retrieve All Players**

  Use the `get_players` query call to retrieve a list of all players.

- **Retrieve Player by ID**

  Use the `get_player_by_id` query call with the player's ID to get detailed information.

### Transfer Management

- **Create a Transfer**

  Use the `create_transfer` update call to record a new transfer. Provide the transfer details in the `TransferPayload`.

  ```json
  {
    "player_id": 1,
    "from_team": "Warriors",
    "to_team": "Tigers",
    "transfer_fee": 500000,
    "transfer_date": 1693017600,
    "contract_duration": 2
  }
  ```

- **Retrieve All Transfers**

  Use the `get_transfers` query call to retrieve a list of all transfers.

- **Retrieve Transfer by ID**

  Use the `get_transfer_by_id` query call with the transfer's ID to get detailed information.

### Transfer Offers

- **Create a Transfer Offer**

  Use the `create_transfer_offer` update call to make a new transfer offer. Provide the offer details in the `TransferOfferPayload`.

  ```json
  {
    "player_id": 1,
    "from_team": "Tigers",
    "to_team": "Eagles",
    "offer_amount": 700000
  }
  ```

- **Retrieve All Transfer Offers**

  Use the `get_transfer_offers` query call to retrieve a list of all transfer offers.

- **Retrieve Transfer Offer by ID**

  Use the `get_transfer_offer_by_id` query call with the offer's ID to get detailed information.

- **Accept or Reject a Transfer Offer**

  Use `accept_transfer_offer` or `reject_transfer_offer` update calls with the offer ID to accept or reject a transfer offer.

## Error Handling

- **Invalid Payload**: Ensure all required fields are provided and valid.
- **Not Found**: Check if the provided ID exists in the system.
- **Error**: Review the error message for specific issues related to the operation.

---

Thank you for using the Rugby League Player Transfer Management System!
```

