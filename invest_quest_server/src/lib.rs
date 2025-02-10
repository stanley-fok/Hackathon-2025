pub mod rewards;
use argon2::{Argon2, PasswordHasher, PasswordVerifier, password_hash::{Salt, rand_core::OsRng, SaltString}, PasswordHash};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Account {
    username: String,
    hash: String,
    balance: u64,
    email: String
}

impl Account {
    pub fn new(username: &str, password: &[u8], email: &str) -> Self {
        let salt = SaltString::generate(&mut OsRng);
        let argon_2 = Argon2::default();
        let hash = String::from(
            argon_2
                .hash_password(password, &salt)
                .unwrap()
                .serialize()
                .as_str()
        );
        Account {
            username: String::from(username),
            hash,
            balance: 500000,
            email: String::from(email)
        }
    }

    fn deposit(&mut self, money: u64) -> Result<(), DepositError> {
        self.balance += money;
        Ok(())
    }

    fn withdraw(&mut self, money: u64) -> Result<u64, WithdrawError> {
        if self.balance>=money {
            self.balance -= money;
            Ok(money)
        } else {
            Err(WithdrawError::InsufficientBalance)
        }
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn verify_password(&self, password: &String) -> bool {
        Argon2::default().verify_password(password.as_bytes(), &PasswordHash::new(&self.hash).unwrap()).is_ok()
    }
}

#[derive(serde::Serialize)]
pub struct AccountMessage<'a> {
    pub name: &'a str,
    pub balance: u64
}


enum DepositError {
}

enum WithdrawError {
    InsufficientBalance
}

pub trait SavingsVehicle {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError>;
}

#[derive(Debug)]
pub enum ProjectionError {
    InterestTooShort
}

struct BankAccount<T: SavingsVehicle> {
    balance: u64,
    savings_vehicle: T
}

impl<T: SavingsVehicle> BankAccount<T> {
    fn new(balance: u64, savings_vehicle: T) -> Self {
        Self {
            balance,
            savings_vehicle
        }
    }

    fn project(self, period: usize) -> Result<Vec<u64>, ProjectionError> {
        self.savings_vehicle.project(self.balance, period)
    }
}

pub struct CurrentAccount;

impl SavingsVehicle for CurrentAccount {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError> {
        Ok(std::iter::repeat(balance).take(period).collect())
    }
}

pub struct SavingsAccount {
    interest_rate: Vec<f64>
}

impl SavingsAccount {
    pub fn new() -> SavingsAccount {
        SavingsAccount {
            interest_rate: (0_u64..121).map(|x| (x as f64/(30.0*120000000.0)).sqrt()).collect()
        }
    }
}

impl SavingsVehicle for SavingsAccount {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError> {
        let mut projection = Vec::with_capacity(period);
        projection.push(
                ((balance as f64) *
                (1.0+self.interest_rate.get(0).ok_or(ProjectionError::InterestTooShort)?))
                as u64);
        for i in 1..period {
            projection.push(
                ((projection[i-1] as f64) *
                (1.0 + self.interest_rate.get(i).ok_or(ProjectionError::InterestTooShort)?))
                as u64);
        }
        Ok(projection)
    }
}