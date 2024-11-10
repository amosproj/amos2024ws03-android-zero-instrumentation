// maybe move this file some where else
// diese Datei soll das arbeiten mit oneof feldern in protobuf messages erleichtern

// use crate::ziofa::EbpfStreamObject;
// use crate::ziofa::*;
// 
// pub trait EbpfStreamObjectBuilder{
//     fn create_stream_object(self) -> EbpfStreamObject;
// }
// 
// impl EbpfStreamObjectBuilder for ConcreteEbpfStreamObject1 {
//     fn create_stream_object(self) -> EbpfStreamObject {
//         EbpfStreamObject {
//             concrete: Some(ebpf_stream_object::Concrete::Concrete1(self)),
//         }
//     }
// }
// 
// impl EbpfStreamObjectBuilder for ConcreteEbpfStreamObject2 {
//     fn create_stream_object(self) -> EbpfStreamObject {
//         EbpfStreamObject {
//             concrete: Some(ebpf_stream_object::Concrete::Concrete2(self)),
//         }
//     }
// }
// 
// 
