use super::aiserver::v1::StreamChatResponse;
use flate2::read::GzDecoder;
use prost::Message;
use std::io::Read;

use super::error::{ChatError, StreamError};

// 解压gzip数据
fn decompress_gzip(data: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();

    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Some(decompressed),
        Err(_) => {
            // println!("gzip解压失败: {}", e);
            None
        }
    }
}

pub enum StreamMessage {
    // 未完成
    Incomplete,
    // 调试
    Debug(String),
    // 流开始标志 b"\0\0\0\0\0"
    StreamStart,
    // 消息内容
    Content(Vec<String>),
    // 流结束标志 b"\x02\0\0\0\x02{}"
    StreamEnd,
}

pub fn parse_stream_data(data: &[u8]) -> Result<StreamMessage, StreamError> {
    if data.len() < 5 {
        return Err(StreamError::DataLengthLessThan5);
    }

    // 检查是否为流开始标志
    // if data == b"\0\0\0\0\0" {
    //     return Ok(StreamMessage::StreamStart);
    // }

    // 检查是否为流结束标志
    // if data == b"\x02\0\0\0\x02{}" {
    //     return Ok(StreamMessage::StreamEnd);
    // }

    let mut messages = Vec::new();
    let mut offset = 0;

    while offset + 5 <= data.len() {
        // 获取消息类型和长度
        let msg_type = data[offset];
        let msg_len = u32::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
        ]) as usize;

        // 流开始
        if msg_type == 0 && msg_len == 0 {
            return Ok(StreamMessage::StreamStart);
        }

        // 检查剩余数据长度是否足够
        if offset + 5 + msg_len > data.len() {
            return Ok(StreamMessage::Incomplete);
        }

        let msg_data = &data[offset + 5..offset + 5 + msg_len];

        match msg_type {
            // 文本消息
            0 => {
                if let Ok(response) = StreamChatResponse::decode(msg_data) {
                    // crate::debug_println!("[text] StreamChatResponse: {:?}", response);
                    if !response.text.is_empty() {
                        messages.push(response.text);
                    } else {
                        // println!("[text] StreamChatResponse: {:?}", response);
                        return Ok(StreamMessage::Debug(
                            response.filled_prompt.unwrap_or_default(),
                            // response.is_using_slow_request,
                        ));
                    }
                }
            }
            // gzip压缩消息
            1 => {
                if let Some(text) = decompress_gzip(msg_data) {
                    if let Ok(response) = StreamChatResponse::decode(&text[..]) {
                        // crate::debug_println!("[gzip] StreamChatResponse: {:?}", response);
                        if !response.text.is_empty() {
                            messages.push(response.text);
                        } else {
                            // println!("[gzip] StreamChatResponse: {:?}", response);
                            return Ok(StreamMessage::Debug(
                                response.filled_prompt.unwrap_or_default(),
                                // response.is_using_slow_request,
                            ));
                        }
                    }
                }
            }
            // JSON字符串
            2 => {
                if msg_len == 2 {
                    return Ok(StreamMessage::StreamEnd);
                }
                if let Ok(text) = String::from_utf8(msg_data.to_vec()) {
                    // println!("JSON消息: {}", text);
                    if let Ok(error) = serde_json::from_str::<ChatError>(&text) {
                        return Err(StreamError::ChatError(error));
                    }
                    // 未预计
                    // messages.push(text);
                }
            }
            // gzip压缩消息
            3 => {
                if let Some(text) = decompress_gzip(msg_data) {
                    if text.len() == 2 {
                        return Ok(StreamMessage::StreamEnd);
                    }
                    if let Ok(text) = String::from_utf8(text) {
                        // println!("JSON消息: {}", text);
                        if let Ok(error) = serde_json::from_str::<ChatError>(&text) {
                            return Err(StreamError::ChatError(error));
                        }
                        // 未预计
                        // messages.push(text);
                    }
                }
            }
            // 其他类型暂不处理
            t => {
                eprintln!("收到未知消息类型: {}，请尝试联系开发者以获取支持", t);
                crate::debug_println!("消息类型: {}，消息内容: {}", t, hex::encode(msg_data));
            }
        }

        offset += 5 + msg_len;
    }

    if messages.is_empty() {
        Err(StreamError::EmptyMessage)
    } else {
        Ok(StreamMessage::Content(messages))
    }
}

#[test]
fn test_parse_stream_data() {
    // 使用include_str!加载测试数据文件
    let stream_data = include_str!("../../tests/data/stream_data.txt");

    // 将整个字符串按每两个字符分割成字节
    let bytes: Vec<u8> = stream_data
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let hex_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(hex_str, 16).unwrap()
        })
        .collect();

    // 辅助函数：找到下一个消息边界
    fn find_next_message_boundary(bytes: &[u8]) -> usize {
        if bytes.len() < 5 {
            return bytes.len();
        }
        let msg_len = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as usize;
        5 + msg_len
    }

    // 辅助函数：将字节转换为hex字符串
    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join("")
    }

    // 多次解析数据
    let mut offset = 0;
    while offset < bytes.len() {
        let remaining_bytes = &bytes[offset..];
        let msg_boundary = find_next_message_boundary(remaining_bytes);
        let current_msg_bytes = &remaining_bytes[..msg_boundary];
        let hex_str = bytes_to_hex(current_msg_bytes);

        match parse_stream_data(current_msg_bytes) {
            Ok(message) => {
                match message {
                    StreamMessage::Content(messages) => {
                        print!("消息内容 [hex: {}]:", hex_str);
                        for msg in messages {
                            println!(" {}", msg);
                        }
                        offset += msg_boundary;
                    }
                    StreamMessage::Debug(_) => {
                        // println!("调试信息 [hex: {}]: {}", hex_str, prompt);
                        offset += msg_boundary;
                    }
                    StreamMessage::StreamEnd => {
                        println!("流结束 [hex: {}]", hex_str);
                        break;
                    }
                    StreamMessage::StreamStart => {
                        println!("流开始 [hex: {}]", hex_str);
                        offset += msg_boundary;
                    }
                    StreamMessage::Incomplete => {
                        println!("数据不完整 [hex: {}]", hex_str);
                        break;
                    }
                }
            }
            Err(e) => {
                println!("解析错误 [hex: {}]: {}", hex_str, e);
                break;
            }
        }
    }
}
