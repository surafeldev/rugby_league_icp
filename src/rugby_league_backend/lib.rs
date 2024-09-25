#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Type aliases for memory management
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Structure for storing rugby players
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct PlayerProfile {
    id: u64,
    name: String,
    position: String,
    current_team: String, // Team the player is currently playing for
    market_value: u64,    // Market value of the player
    transfer_status: String, // Transfer status (e.g., "available", "transferred")
    contract_until: u64, // Timestamp of when the player's contract expires
    age: u32,
    nationality: String,
    created_at: u64, // Timestamp when the player profile was created
}

// Structure for storing rugby player transfers
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct PlayerTransfer {
    id: u64,
    player_id: u64, // ID of the player being transferred
    from_team: String, // Team from which the player is transferring
    to_team: String, // Team to which the player is transferring
    transfer_fee: u64, // Fee for the transfer
    transfer_date: u64, // Timestamp of the transfer
    contract_duration: u64, // Duration of the contract with the new team
    created_at: u64, // Timestamp when the transfer record was created
}

// Structure for storing transfer offers for players
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct TransferOffer {
    id: u64,
    player_id: u64, // ID of the player for whom the offer is made
    from_team: String, // Team making the offer
    to_team: String, // Team receiving the offer
    offer_amount: u64, // Amount offered for the transfer
    offer_status: String, // Status of the offer (e.g., "pending", "accepted", "rejected")
    created_at: u64, // Timestamp when the offer was made
}

// Implementations for serialization and storage
impl Storable for PlayerProfile {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for PlayerProfile {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for PlayerTransfer {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for PlayerTransfer {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for TransferOffer {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for TransferOffer {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for memory management and data
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static PLAYER_PROFILE_STORAGE: RefCell<StableBTreeMap<u64, PlayerProfile, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static PLAYER_TRANSFER_STORAGE: RefCell<StableBTreeMap<u64, PlayerTransfer, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static TRANSFER_OFFER_STORAGE: RefCell<StableBTreeMap<u64, TransferOffer, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

// Payloads for creating and updating player profiles and transfers
#[derive(candid::CandidType, Deserialize, Serialize)]
struct PlayerProfilePayload {
    name: String,
    position: String,
    current_team: String,
    market_value: u64,
    contract_until: u64,
    age: u32,
    nationality: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct PlayerTransferPayload {
    player_id: u64,
    from_team: String,
    to_team: String,
    transfer_fee: u64,
    transfer_date: u64,
    contract_duration: u64,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct TransferOfferPayload {
    player_id: u64,
    from_team: String,
    to_team: String,
    offer_amount: u64,
}

// Enumeration for different types of messages
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Function to create a new player profile
#[ic_cdk::update]
fn create_player_profile(payload: PlayerProfilePayload) -> Result<PlayerProfile, Message> {
    if payload.name.is_empty()
        || payload.position.is_empty()
        || payload.current_team.is_empty()
        || payload.market_value == 0
    {
        return Err(Message::InvalidPayload(
            "Ensure 'name', 'position', 'current_team', and 'market_value' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let player_profile = PlayerProfile {
        id,
        name: payload.name,
        position: payload.position,
        current_team: payload.current_team,
        market_value: payload.market_value,
        transfer_status: "available".to_string(),
        contract_until: payload.contract_until,
        age: payload.age,
        nationality: payload.nationality,
        created_at: current_time(),
    };
    PLAYER_PROFILE_STORAGE.with(|storage| storage.borrow_mut().insert(id, player_profile.clone()));
    Ok(player_profile)
}

// Function to retrieve all player profiles
#[ic_cdk::query]
fn get_player_profiles() -> Result<Vec<PlayerProfile>, Message> {
    PLAYER_PROFILE_STORAGE.with(|storage| {
        let player_profiles: Vec<PlayerProfile> = storage
            .borrow()
            .iter()
            .map(|(_, player_profile)| player_profile.clone())
            .collect();

        if player_profiles.is_empty() {
            Err(Message::NotFound("No player profiles found".to_string()))
        } else {
            Ok(player_profiles)
        }
    })
}

// Function to retrieve a player profile by ID
#[ic_cdk::query]
fn get_player_profile_by_id(id: u64) -> Result<PlayerProfile, Message> {
    PLAYER_PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, player_profile)| player_profile.id == id)
            .map(|(_, player_profile)| player_profile.clone())
            .ok_or(Message::NotFound("Player profile not found".to_string()))
    })
}

// Function to retrieve player profiles by team
#[ic_cdk::query]
fn get_player_profiles_by_team(team_name: String) -> Result<Vec<PlayerProfile>, Message> {
    PLAYER_PROFILE_STORAGE.with(|storage| {
        let player_profiles: Vec<PlayerProfile> = storage
            .borrow()
            .iter()
            .filter(|(_, player_profile)| player_profile.current_team == team_name)
            .map(|(_, player_profile)| player_profile.clone())
            .collect();

        if player_profiles.is_empty() {
            Err(Message::NotFound("No players found for the specified team".to_string()))
        } else {
            Ok(player_profiles)
        }
    })
}

// Function to create a new player transfer
#[ic_cdk::update]
fn create_player_transfer(payload: PlayerTransferPayload) -> Result<PlayerTransfer, Message> {
    // Validate the payload
    if payload.transfer_fee == 0 || payload.contract_duration == 0 {
        return Err(Message::InvalidPayload(
            "Ensure 'transfer_fee' and 'contract_duration' are provided.".to_string(),
        ));
    }

    // Ensure the player is not transferring to the same team
    if payload.from_team == payload.to_team {
        return Err(Message::InvalidPayload(
            "Transfer team must be different from the current team.".to_string(),
        ));
    }

    // Retrieve player profile to validate transfer
    let player_profile = PLAYER_PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, player_profile)| player_profile.id == payload.player_id)
            .map(|(_, player_profile)| player_profile.clone())
    });

    if player_profile.is_none() {
        return Err(Message::NotFound("Player profile not found".to_string()));
    }

    let player_profile = player_profile.unwrap();

    // Check if player is available for transfer and is part of the from_team
    if player_profile.transfer_status != "available" || player_profile.current_team != payload.from_team {
        return Err(Message::Error(
            "Player is not available for transfer or is not a member of the from_team.".to_string(),
        ));
    }

    // Player must not have been transferred already
    if player_profile.transfer_status == "transferred" {
        return Err(Message::Error(
            "Player has already been transferred.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let player_transfer = PlayerTransfer {
        id,
        player_id: payload.player_id,
        from_team: payload.from_team.clone(),
        to_team: payload.to_team.clone(),
        transfer_fee: payload.transfer_fee,
        transfer_date: payload.transfer_date,
        contract_duration: payload.contract_duration,
        created_at: current_time(),
    };

    PLAYER_TRANSFER_STORAGE.with(|storage| storage.borrow_mut().insert(id, player_transfer.clone()));

    // Update player profile to reflect transfer status and new team
    let mut updated_player_profile = player_profile.clone();
    updated_player_profile.transfer_status = "transferred".to_string();
    updated_player_profile.current_team = payload.to_team.clone();
    updated_player_profile.contract_until = payload.transfer_date + payload.contract_duration;
    PLAYER_PROFILE_STORAGE.with(|storage| storage.borrow_mut().insert(player_profile.id, updated_player_profile));

    Ok(player_transfer)
}

// Function to retrieve all player transfers
#[ic_cdk::query]
fn get_player_transfers() -> Result<Vec<PlayerTransfer>, Message> {
    PLAYER_TRANSFER_STORAGE.with(|storage| {
        let player_transfers: Vec<PlayerTransfer> = storage
            .borrow()
            .iter()
            .map(|(_, player_transfer)| player_transfer.clone())
            .collect();

        if player_transfers.is_empty() {
            Err(Message::NotFound("No player transfers found".to_string()))
        } else {
            Ok(player_transfers)
        }
    })
}

// Function to retrieve a player transfer by ID
#[ic_cdk::query]
fn get_player_transfer_by_id(id: u64) -> Result<PlayerTransfer, Message> {
    PLAYER_TRANSFER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, player_transfer)| player_transfer.id == id)
            .map(|(_, player_transfer)| player_transfer.clone())
            .ok_or(Message::NotFound("Player transfer not found".to_string()))
    })
}

// Function to create a new transfer offer
#[ic_cdk::update]
fn create_transfer_offer(payload: TransferOfferPayload) -> Result<TransferOffer, Message> {
    if payload.offer_amount == 0 {
        return Err(Message::InvalidPayload(
            "Offer amount must be greater than 0.".to_string(),
        ));
    }

    let player_profile = PLAYER_PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, player_profile)| player_profile.id == payload.player_id)
            .map(|(_, player_profile)| player_profile.clone())
    });

    if player_profile.is_none() {
        return Err(Message::NotFound("Player profile not found".to_string()));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let transfer_offer = TransferOffer {
        id,
        player_id: payload.player_id,
        from_team: payload.from_team.clone(),
        to_team: payload.to_team.clone(),
        offer_amount: payload.offer_amount,
        offer_status: "pending".to_string(),
        created_at: current_time(),
    };

    TRANSFER_OFFER_STORAGE.with(|storage| storage.borrow_mut().insert(id, transfer_offer.clone()));
    Ok(transfer_offer)
}

// Function to retrieve all transfer offers
#[ic_cdk::query]
fn get_transfer_offers() -> Result<Vec<TransferOffer>, Message> {
    TRANSFER_OFFER_STORAGE.with(|storage| {
        let transfer_offers: Vec<TransferOffer> = storage
            .borrow()
            .iter()
            .map(|(_, transfer_offer)| transfer_offer.clone())
            .collect();

        if transfer_offers.is_empty() {
            Err(Message::NotFound("No transfer offers found".to_string()))
        } else {
            Ok(transfer_offers)
        }
    })
}

// Function to retrieve a transfer offer by ID
#[ic_cdk::query]
fn get_transfer_offer_by_id(id: u64) -> Result<TransferOffer, Message> {
    TRANSFER_OFFER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, transfer_offer)| transfer_offer.id == id)
            .map(|(_, transfer_offer)| transfer_offer.clone())
            .ok_or(Message::NotFound("Transfer offer not found".to_string()))
    })
}

// Function to accept a transfer offer
#[ic_cdk::update]
fn accept_transfer_offer(id: u64) -> Result<Message, Message> {
    let offer = TRANSFER_OFFER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, transfer_offer)| transfer_offer.id == id)
            .map(|(_, transfer_offer)| transfer_offer.clone())
    });

    if offer.is_none() {
        return Err(Message::NotFound("Transfer offer not found".to_string()));
    }

    let mut offer = offer.unwrap();

    if offer.offer_status != "pending" {
        return Err(Message::Error(
            "Only pending offers can be accepted.".to_string(),
        ));
    }

    let player_profile = PLAYER_PROFILE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, player_profile)| player_profile.id == offer.player_id)
            .map(|(_, player_profile)| player_profile.clone())
    });

    if player_profile.is_none() {
        return Err(Message::NotFound("Player profile not found".to_string()));
    }

    let player_profile = player_profile.unwrap();

    // Update offer status to accepted
    offer.offer_status = "accepted".to_string();
    TRANSFER_OFFER_STORAGE.with(|storage| storage.borrow_mut().insert(id, offer.clone()));

    // Create a new player transfer
    let transfer_id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let player_transfer = PlayerTransfer {
        id: transfer_id,
        player_id: offer.player_id,
        from_team: offer.from_team.clone(),
        to_team: offer.to_team.clone(),
        transfer_fee: offer.offer_amount,
        transfer_date: current_time(),
        contract_duration: player_profile.contract_until - current_time(), // Assume remaining contract duration
        created_at: current_time(),
    };

    PLAYER_TRANSFER_STORAGE.with(|storage| storage.borrow_mut().insert(transfer_id, player_transfer.clone()));

    // Update player profile to reflect the new team and transfer status
    let mut updated_player_profile = player_profile.clone();
    updated_player_profile.transfer_status = "transferred".to_string();
    updated_player_profile.current_team = offer.to_team.clone();
    updated_player_profile.contract_until = current_time() + player_transfer.contract_duration;
    PLAYER_PROFILE_STORAGE.with(|storage| storage.borrow_mut().insert(player_profile.id, updated_player_profile));

    Ok(Message::Success(
        "Transfer offer accepted and player transferred.".to_string(),
    ))
}

// Function to reject a transfer offer
#[ic_cdk::update]
fn reject_transfer_offer(id: u64) -> Result<Message, Message> {
    let offer = TRANSFER_OFFER_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, transfer_offer)| transfer_offer.id == id)
            .map(|(_, transfer_offer)| transfer_offer.clone())
    });

    if offer.is_none() {
        return Err(Message::NotFound("Transfer offer not found".to_string()));
    }

    let mut offer = offer.unwrap();

    if offer.offer_status != "pending" {
        return Err(Message::Error(
            "Only pending offers can be rejected.".to_string(),
        ));
    }

    // Update offer status to rejected
    offer.offer_status = "rejected".to_string();
    TRANSFER_OFFER_STORAGE.with(|storage| storage.borrow_mut().insert(id, offer.clone()));

    Ok(Message::Success("Transfer offer rejected.".to_string()))
}

// Utility function to get the current time
fn current_time() -> u64 {
    time()
}

ic_cdk::export_candid!();
