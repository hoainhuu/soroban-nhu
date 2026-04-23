#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MealCreditError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InsufficientBalance = 5,
}

#[contract]
pub struct CampusMealCreditContract;

#[contractimpl]
impl CampusMealCreditContract {
    // Khởi tạo contract, chỉ gọi 1 lần
    pub fn init(env: Env, admin: Address ) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, MealCreditError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().extend_ttl(1000, 10000);
    }

    // Lấy admin hiện tại
    pub fn get_admin(env: Env) -> Address {
        Self::require_initialized(&env);
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap()
    }

    // Admin nạp credit cho sinh viên
    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::require_initialized(&env);
        Self::require_positive_amount(&env, amount);

        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        let current_balance = Self::read_balance(&env, to.clone());
        let new_balance = current_balance + amount;

        let key = DataKey::Balance(to);
        env.storage().persistent().set(&key, &new_balance);
        env.storage().persistent().extend_ttl(&key, 1000, 10000);
    }

    // Admin trừ credit của sinh viên khi thanh toán ở căn-tin
    pub fn spend(env: Env, user: Address, amount: i128) {
        Self::require_initialized(&env);
        Self::require_positive_amount(&env, amount);

        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        let current_balance = Self::read_balance(&env, user.clone());
        if current_balance < amount {
            panic_with_error!(&env, MealCreditError::InsufficientBalance);
        }

        let new_balance = current_balance - amount;
        let key = DataKey::Balance(user);
        env.storage().persistent().set(&key, &new_balance);
        env.storage().persistent().extend_ttl(&key, 1000, 10000);
    }

    // Sinh viên tự dùng credit của chính mình
    pub fn spend_my_credit(env: Env, user: Address, amount: i128) {
        Self::require_initialized(&env);
        Self::require_positive_amount(&env, amount);

        user.require_auth();

        let current_balance = Self::read_balance(&env, user.clone());
        if current_balance < amount {
            panic_with_error!(&env, MealCreditError::InsufficientBalance);
        }

        let new_balance = current_balance - amount;
        let key = DataKey::Balance(user);
        env.storage().persistent().set(&key, &new_balance);
        env.storage().persistent().extend_ttl(&key, 1000, 10000);
    }

    // Chuyển credit giữa 2 người dùng
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        Self::require_initialized(&env);
        Self::require_positive_amount(&env, amount);

        from.require_auth();

        let from_balance = Self::read_balance(&env, from.clone());
        if from_balance < amount {
            panic_with_error!(&env, MealCreditError::InsufficientBalance);
        }

        let to_balance = Self::read_balance(&env, to.clone());

        let from_key = DataKey::Balance(from);
        let to_key = DataKey::Balance(to);

        env.storage()
            .persistent()
            .set(&from_key, &(from_balance - amount));
        env.storage()
            .persistent()
            .set(&to_key, &(to_balance + amount));

        env.storage().persistent().extend_ttl(&from_key, 1000, 10000);
        env.storage().persistent().extend_ttl(&to_key, 1000, 10000);
    }

    // Xem số dư
    pub fn balance_of(env: Env, user: Address) -> i128 {
        Self::require_initialized(&env);
        Self::read_balance(&env, user)
    }

    // Đổi admin
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_initialized(&env);

        let admin = Self::get_admin(env.clone());
        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.storage().instance().extend_ttl(1000, 10000);
    }

    fn require_initialized(env: &Env) {
        let initialized = env
            .storage()
            .instance()
            .get::<_, bool>(&DataKey::Initialized)
            .unwrap_or(false);

        if !initialized {
            panic_with_error!(env, MealCreditError::NotInitialized);
        }
    }

    fn require_positive_amount(env: &Env, amount: i128) {
        if amount <= 0 {
            panic_with_error!(env, MealCreditError::InvalidAmount);
        }
    }

    fn read_balance(env: &Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user))
            .unwrap_or(0)
    }
}