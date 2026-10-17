use crate::windows::native_abi::PropRecipe;
use crate::windows::provider::provider;

pub fn world_entity_register_prop_recipes(recipes: &[PropRecipe]) -> Result<(), String> {
    let Some(provider) = provider() else {
        return Err("KFC Runtime provider unavailable".into());
    };
    let accepted =
        unsafe { (provider.world_entity_register_prop_recipes)(recipes.as_ptr(), recipes.len()) };
    if accepted {
        Ok(())
    } else {
        Err("native prop recipe registration was rejected".into())
    }
}
