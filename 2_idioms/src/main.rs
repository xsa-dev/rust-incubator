use std::collections::{hash_map::Entry, BTreeMap, HashMap};
use std::num::NonZeroU32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Product {
    name: String,
    price: NonZeroU32,
}

impl Product {
    pub fn new(name: impl Into<String>, price: NonZeroU32) -> Self {
        Self {
            name: name.into(),
            price,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn price(&self) -> NonZeroU32 {
        self.price
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Coin {
    One,
    Two,
    Five,
    Ten,
    Twenty,
    Fifty,
}

impl Coin {
    pub const ALL: [Coin; 6] = [
        Coin::One,
        Coin::Two,
        Coin::Five,
        Coin::Ten,
        Coin::Twenty,
        Coin::Fifty,
    ];

    pub const fn value(self) -> u32 {
        match self {
            Coin::One => 1,
            Coin::Two => 2,
            Coin::Five => 5,
            Coin::Ten => 10,
            Coin::Twenty => 20,
            Coin::Fifty => 50,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StockError {
    ZeroQuantity,
    ExceedsCapacity { available: usize, requested: usize },
    PriceMismatch { expected: u32, found: u32 },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PurchaseError {
    UnknownProduct,
    OutOfStock,
    InsufficientPayment { price: u32, paid: u32 },
    CannotProvideChange { change: u32 },
}

#[derive(Debug)]
struct Slot {
    product: Product,
    quantity: u32,
}

#[derive(Debug)]
pub struct VendingMachine {
    capacity: usize,
    slots: HashMap<String, Slot>,
    coins: BTreeMap<Coin, u32>,
}

impl VendingMachine {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            slots: HashMap::new(),
            coins: BTreeMap::new(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn total_items(&self) -> usize {
        self.slots
            .values()
            .map(|slot| slot.quantity as usize)
            .sum()
    }

    pub fn available_capacity(&self) -> usize {
        self.capacity.saturating_sub(self.total_items())
    }

    pub fn restock(&mut self, product: Product, quantity: u32) -> Result<(), StockError> {
        if quantity == 0 {
            return Err(StockError::ZeroQuantity);
        }

        let requested = quantity as usize;
        let available = self.available_capacity();
        if requested > available {
            return Err(StockError::ExceedsCapacity { available, requested });
        }

        match self.slots.entry(product.name().to_owned()) {
            Entry::Occupied(mut entry) => {
                let existing_price = entry.get().product.price.get();
                if existing_price != product.price.get() {
                    return Err(StockError::PriceMismatch {
                        expected: existing_price,
                        found: product.price.get(),
                    });
                }
                entry.get_mut().quantity += quantity;
            }
            Entry::Vacant(entry) => {
                entry.insert(Slot { product, quantity });
            }
        }

        Ok(())
    }

    pub fn add_change(&mut self, coins: impl IntoIterator<Item = Coin>) {
        for coin in coins {
            *self.coins.entry(coin).or_insert(0) += 1;
        }
    }

    pub fn purchase(
        &mut self,
        name: &str,
        payment: impl IntoIterator<Item = Coin>,
    ) -> Result<(Product, Vec<Coin>), PurchaseError> {
        let price = {
            let slot = self
                .slots
                .get(name)
                .ok_or(PurchaseError::UnknownProduct)?;
            if slot.quantity == 0 {
                return Err(PurchaseError::OutOfStock);
            }
            slot.product.price.get()
        };

        let payment_coins: Vec<Coin> = payment.into_iter().collect();
        let paid: u32 = payment_coins.iter().map(|coin| coin.value()).sum();

        if paid < price {
            return Err(PurchaseError::InsufficientPayment { price, paid });
        }

        let change_amount = paid - price;

        let mut combined = self.coins.clone();
        for coin in &payment_coins {
            *combined.entry(*coin).or_insert(0) += 1;
        }

        let change = Self::calculate_change(&combined, change_amount)
            .ok_or(PurchaseError::CannotProvideChange {
                change: change_amount,
            })?;

        for coin in payment_coins {
            *self.coins.entry(coin).or_insert(0) += 1;
        }

        Self::deduct_change(&mut self.coins, &change);

        let mut remove_slot = false;
        let product = {
            let slot = self
                .slots
                .get_mut(name)
                .expect("slot must exist while completing purchase");
            slot.quantity -= 1;
            if slot.quantity == 0 {
                remove_slot = true;
            }
            slot.product.clone()
        };

        if remove_slot {
            self.slots.remove(name);
        }

        Ok((product, change))
    }

    fn calculate_change(coins: &BTreeMap<Coin, u32>, amount: u32) -> Option<Vec<Coin>> {
        if amount == 0 {
            return Some(Vec::new());
        }

        let mut remaining = amount;
        let mut result = Vec::new();

        for coin in Coin::ALL.iter().rev() {
            let value = coin.value();
            let available = *coins.get(coin).unwrap_or(&0);
            if available == 0 || value > remaining {
                continue;
            }

            let usable = (remaining / value).min(available);
            if usable == 0 {
                continue;
            }

            result.extend(std::iter::repeat(*coin).take(usable as usize));
            remaining -= value * usable;

            if remaining == 0 {
                break;
            }
        }

        if remaining == 0 {
            Some(result)
        } else {
            None
        }
    }

    fn deduct_change(coins: &mut BTreeMap<Coin, u32>, change: &[Coin]) {
        let mut zeroed = Vec::new();
        for coin in change {
            if let Some(entry) = coins.get_mut(coin) {
                *entry -= 1;
                if *entry == 0 {
                    zeroed.push(*coin);
                }
            }
        }

        for coin in zeroed {
            coins.remove(&coin);
        }
    }
}

fn main() {
    let mut machine = VendingMachine::new(5);

    let cola = Product::new("Cola", NonZeroU32::new(45).expect("price must be non-zero"));
    machine
        .restock(cola, 2)
        .expect("failed to restock the machine");

    machine.add_change([
        Coin::Twenty,
        Coin::Twenty,
        Coin::Five,
        Coin::Two,
        Coin::Two,
    ]);

    let payment = [Coin::Fifty];
    match machine.purchase("Cola", payment) {
        Ok((product, change)) => {
            println!(
                "Enjoy your {}! Change: {:?}",
                product.name(),
                change
            );
        }
        Err(err) => {
            println!("Cannot complete purchase: {:?}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purchase_with_change() {
        let mut machine = VendingMachine::new(3);
        let soda = Product::new("Soda", NonZeroU32::new(45).unwrap());
        machine.restock(soda, 2).unwrap();
        machine.add_change([Coin::Twenty, Coin::Twenty, Coin::Five]);

        let (product, change) = machine.purchase("Soda", [Coin::Fifty]).unwrap();
        assert_eq!(product.name(), "Soda");
        assert_eq!(product.price().get(), 45);
        assert_eq!(change, vec![Coin::Five]);
        assert_eq!(machine.total_items(), 1);
    }

    #[test]
    fn insufficient_payment_is_rejected() {
        let mut machine = VendingMachine::new(1);
        let snack = Product::new("Snack", NonZeroU32::new(20).unwrap());
        machine.restock(snack, 1).unwrap();

        let err = machine.purchase("Snack", [Coin::Ten]).unwrap_err();
        assert_eq!(
            err,
            PurchaseError::InsufficientPayment {
                price: 20,
                paid: 10
            }
        );
    }

    #[test]
    fn cannot_provide_change() {
        let mut machine = VendingMachine::new(2);
        let water = Product::new("Water", NonZeroU32::new(30).unwrap());
        machine.restock(water, 1).unwrap();
        machine.add_change([Coin::Ten]);

        let err = machine.purchase("Water", [Coin::Fifty]).unwrap_err();
        assert_eq!(
            err,
            PurchaseError::CannotProvideChange { change: 20 }
        );
    }

    #[test]
    fn restock_respects_capacity() {
        let mut machine = VendingMachine::new(1);
        let snack = Product::new("Snack", NonZeroU32::new(10).unwrap());
        machine.restock(snack.clone(), 1).unwrap();
        let err = machine.restock(snack, 1).unwrap_err();
        assert_eq!(
            err,
            StockError::ExceedsCapacity {
                available: 0,
                requested: 1
            }
        );
    }

    #[test]
    fn restock_rejects_different_price() {
        let mut machine = VendingMachine::new(2);
        let snack = Product::new("Snack", NonZeroU32::new(10).unwrap());
        machine.restock(snack.clone(), 1).unwrap();

        let err = machine
            .restock(Product::new("Snack", NonZeroU32::new(20).unwrap()), 1)
            .unwrap_err();

        assert_eq!(
            err,
            StockError::PriceMismatch {
                expected: 10,
                found: 20
            }
        );
    }
}
