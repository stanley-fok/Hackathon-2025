struct Account {
    username: String,
    salt: String,
    hash: Vec<u8>,
    balance: u64
}

impl Account {
    fn new(username: &str, password: &str) -> Self {
        //todo: proper authentication
        let salt = String::from("placeholder");
        Account {
            username: String::from(username),
            salt: salt.clone(),
            hash: (String::from_utf8(password.into()).unwrap()+&salt).into(),
            balance: 0
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
}

enum DepositError {
}

enum WithdrawError {
    InsufficientBalance
}