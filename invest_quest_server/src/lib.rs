pub mod rewards;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Account {
    username: String,
    salt: String,
    hash: Vec<u8>,
    balance: u64,
    email: String
}

impl Account {
    pub fn new(username: &str, password: &str, email: &str) -> Self {
        //todo: proper authentication
        let salt = String::from("placeholder");
        Account {
            username: String::from(username),
            salt: salt.clone(),
            hash: (String::from_utf8(password.into()).unwrap()+&salt).into(),
            balance: 0,
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

    pub fn get_hash(&self) -> &[u8] {
        &self.hash
    }

    pub fn get_salt(&self) -> &str {
        &self.salt
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}

enum DepositError {
}

enum WithdrawError {
    InsufficientBalance
}

trait SavingsVehicle {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError>;
}

enum ProjectionError {
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

struct CurrentAccount;

impl SavingsVehicle for CurrentAccount {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError> {
        Ok(std::iter::repeat(balance).take(period).collect())
    }
}

struct SavingsAccount {
    interest_rate: Vec<f64>
}

impl SavingsVehicle for SavingsAccount {
    fn project(&self, balance: u64, period: usize) -> Result<Vec<u64>, ProjectionError> {
        let mut projection = Vec::with_capacity(period);
        projection.push(
                ((balance as f64) *
                (1.0+self.interest_rate.get(0).ok_or(ProjectionError::InterestTooShort)?))
                as u64);
        for i in 1..self.interest_rate.len() {
            projection.push(
                ((projection[i-1] as f64) *
                (1.0 + self.interest_rate.get(i).ok_or(ProjectionError::InterestTooShort)?))
                as u64);
        }
        Ok(projection)
    }
}