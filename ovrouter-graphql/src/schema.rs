//! 这里配置 graphQL 输入输出
//! main.rs 几乎不用修改了

use juniper::FieldResult;
use juniper::RootNode;

use data_struct;

use netdb::Database;

// domain 为数据结构 graphQL 可以获取的数据结构
// database为数据源
//
// ovrouter 应该是
//      domain:     Info,
//      db:         info_arc: Arc<Mutex<Info>>,
//
pub struct Data {
    pub domain:data_struct::Domain,
    pub db:Database,
}
impl Data {
    // 每次web 获取数据的时候 通过db 创建一个包含数据的实例
    pub fn new() -> Self {
        Data{
            domain:data_struct::Domain::new(),
            db:Database::new(),
        }
    }
}

// graphql 的宏
// field 请求的数据结构体名称
// 返回类型： 数据结构体
///
/// example:
/// #[derive(Clone, Debug, RustcDecodable, RustcEncodable, GraphQLObject)]
///pub struct ScanNode {
///    pub node_info:          Node,
///    pub status:             String,
///    pub multi_ip:           Vec<Node>,
///    pub sub_node:           Vec<SubNode>,
///    pub traceroute:         Vec<String>,
///    pub last_scan_time:     String,
///}

//pub struct QueryRoot;
graphql_object!(Data: () |&self| {
    field scan_node() -> Option<Vec<data_struct::ScanNode>> as "The scan of the character" {
        let domain = self.domain.clone().get_from_db(&self.db, "siteview");
        Some(domain.scan_node)
    }
});

pub struct MutationRoot;

// 从web 传入数据
#[derive(GraphQLInputObject)]
#[graphql(description = "create new data")]
struct NewScanNode {
    ip:String
}
// 处理  从web 传入的数据
graphql_object!(MutationRoot: () |&self| {
    field NewScanNode(&executor, new: NewScanNode) -> FieldResult<NewScanNode> {
        Ok(NewScanNode{
            ip:"123".to_string(),
        })
    }
});


// 以下 创建 graphQL结构
pub type Schema = RootNode<'static, Data, MutationRoot>;

pub fn create_schema() -> Schema {
    Schema::new(Data::new(), MutationRoot{})
}