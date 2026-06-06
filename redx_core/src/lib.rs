struct Item {
    price: u64,
    id: u64,
    in_stock: bool,
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
    #[test]
    fn test_item() {
        let chip: Item = Item {
            price: add(3, 3),
            id: 2390239,
            in_stock: true,
        };
        assert_eq!(chip.price, 6);
        assert_eq!(chip.in_stock, true);
    }
}
