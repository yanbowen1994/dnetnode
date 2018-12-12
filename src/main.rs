pub mod net_tool;
pub mod sys_tool;

fn main() {
    let (res, code) = net_tool::http_get("http://www.baidu.com/");
    println!("{}", res);
    println!("{}", code);
}