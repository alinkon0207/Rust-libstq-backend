#[derive(GraphQLEnum, Deserialize, Serialize, Debug, Clone, PartialEq, DieselTypes)]
#[graphql(name = "AttributeType", description = "Attribute Type")]
pub enum AttributeType {
    #[graphql(description = "String type. Can represent enums, bool, int and strings.")]
    Str,
    #[graphql(description = "Float type.")]
    Float,
}
