/// a clickable experience! its like roblox they call their games "experiences" its so dumb
/// dont touch this btw
#[derive(Clone, Copy)]
pub struct Experience {
    pub name: &'static str,
    pub description: &'static str,
    pub path: &'static str,
    pub background: &'static str,
}

/// add new experiences here
/// theyre just structs
pub const EXPERIENCES: &[Experience] = &[Experience {
    name: "Test",
    background: "images/glitchcity.jpg",
    description: "test experience to make sure everything works",
    path: "/test",
}];

/// also add the pages here as you add experiences
pub mod test;
