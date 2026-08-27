use once_cell::sync::OnceCell;
#[derive(Debug)]
pub struct Const{
    pub headless: bool
}

pub static CONST: OnceCell<Const> = OnceCell::new();
impl Const{
    pub fn new(val: Const){
        CONST.set(val).expect("Failed to set consts");
    }
    pub fn get() -> &'static Const{
        CONST.get().unwrap()
    }
}