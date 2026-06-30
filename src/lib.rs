pub mod logistic_regression;
pub mod utility;
pub mod protocol;
pub mod constants;
pub mod init;
pub mod trusted_initializer;
pub mod computing_party;
pub mod ti_receiver;
//pub mod single_protocol;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
