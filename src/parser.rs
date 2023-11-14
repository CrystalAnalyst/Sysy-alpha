use crate::BasicType;
use crate::NodeType;

#[derive(Clone)]
pub struct Node {
    pub node_type: NodeType,   //NodeType是Ast的节点类型
    pub basci_type: BasicType, //BasicType是SysY语言的基本类型
    pub startpos: usize,       //start是(该)节点在源代码中的起始位置
    pub endpos: usize,         //end是(该)节点在源代码中的结束位置
}
