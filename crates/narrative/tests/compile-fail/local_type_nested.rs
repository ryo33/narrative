#[narrative::story("Story with local types")]
trait ConstWithoutValueStory {
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(ConstWithoutValueStory)]
struct LocalTypeStruct {
    field: LocalTypeEnum,
}

#[derive(Debug, Clone, serde::Serialize)]
#[narrative::local_type_for(ConstWithoutValueStory)]
enum LocalTypeEnum {
    Variant1,
    Variant2(NonLocalType),
}

#[derive(Debug, Clone, serde::Serialize)]
struct NonLocalType;

fn main() {}
