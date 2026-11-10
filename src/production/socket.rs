use super::{Config,log};
use serde_json::Value;
use std::{
 sync::{Arc,mpsc,atomic::{AtomicBool,Ordering}},
 thread,time::Duration,net::TcpStream,io::ErrorKind
};
use tungstenite::{Message,stream::MaybeTlsStream,client::IntoClientRequest,http::HeaderValue};

#[derive(Debug)]
pub enum Event {
 Connected, Ready, Disconnected, JobAvailable, ConfigChanged, PdfToolChanged, UpdateChanged
}
pub struct Connection {
 pub events:mpsc::Receiver<Event>,
 pub outgoing:mpsc::Sender<Value>,
 pub ready:Arc<AtomicBool>
}
fn set_read_timeout(stream:&mut MaybeTlsStream<TcpStream>){
 let duration=Some(Duration::from_millis(750));
 match stream {
  MaybeTlsStream::Plain(tcp)=>{let _=tcp.set_read_timeout(duration);},
  MaybeTlsStream::NativeTls(tls)=>{let _=tls.get_ref().set_read_timeout(duration);},
  _=>{}
 }
}
pub fn start(c:&Config,agent_id:String)->Connection{
 let (sender,events)=mpsc::channel();
 let (outgoing_tx,outgoing_rx)=mpsc::channel::<Value>();
 let ready=Arc::new(AtomicBool::new(false));
 let state=Arc::clone(&ready);
 let url=c.api_base_url.trim_end_matches('/').to_string();
 let token=c.agent_token.clone();
 thread::spawn(move||{
  let endpoint=if url.starts_with("https://"){url.replacen("https://","wss://",1)}
               else{url.replacen("http://","ws://",1)};
  let target=format!("{endpoint}/api/print-agent/socket");
  let mut reconnect_delay=5u64;
  loop {
   let result=(||->Result<(),String>{
    let mut request=target.as_str().into_client_request().map_err(|e|e.to_string())?;
    request.headers_mut().insert("Authorization",HeaderValue::from_str(&format!("Bearer {token}")).map_err(|e|e.to_string())?);
    request.headers_mut().insert("X-Print-Agent-Id",HeaderValue::from_str(&agent_id).map_err(|e|e.to_string())?);
    let (mut socket,_) = tungstenite::connect(request).map_err(|e|e.to_string())?;
    set_read_timeout(socket.get_mut());
    reconnect_delay=5;
    log("WebSocket Rust đã kết nối; đang xác nhận giao thức");
    let _=sender.send(Event::Connected);
    loop{
     // Chỉ gửi lệnh qua WSS sau khi Worker xác nhận protocol v2.
     if state.load(Ordering::Relaxed){
      while let Ok(message)=outgoing_rx.try_recv(){
       socket.send(Message::Text(message.to_string().into())).map_err(|e|e.to_string())?;
      }
     }
     match socket.read(){
      Ok(message)=>{
       if message.is_text(){
        if let Ok(text)=message.to_text(){
         if let Ok(v)=serde_json::from_str::<Value>(text){
          match v["type"].as_str().unwrap_or("") {
           "protocol" if v["version"].as_u64()==Some(2)=>{
            state.store(true,Ordering::Relaxed);
            log("WebSocket protocol v2 sẵn sàng; tắt HTTP polling thường xuyên");
            let _=sender.send(Event::Ready);
           },
           "job-available"=>{let _=sender.send(Event::JobAvailable);},
           "config-changed"=>{let _=sender.send(Event::ConfigChanged);},
           "pdf-tool-changed"=>{let _=sender.send(Event::PdfToolChanged);},
           "update-changed"=>{let _=sender.send(Event::UpdateChanged);},
           "heartbeat-error"=>log("WebSocket heartbeat bị Worker từ chối; kiểm tra cấu hình"),
           _=>{}
          }
         }
        }
       }else if message.is_ping(){
        socket.send(Message::Pong(message.into_data())).map_err(|e|e.to_string())?;
       }else if message.is_close(){return Ok(())}
      },
      Err(tungstenite::Error::Io(ref e)) if matches!(e.kind(),ErrorKind::WouldBlock|ErrorKind::TimedOut)=>{},
      Err(e)=>return Err(e.to_string())
     }
    }
   })();
   state.store(false,Ordering::Relaxed);
   let _=sender.send(Event::Disconnected);
   if let Err(e)=result{log(&format!("WebSocket mất kết nối, bật HTTP dự phòng: {e}"))}
   // Bỏ heartbeat chưa gửi khi đứt mạng, tránh gửi trạng thái cũ sau reconnect.
   while outgoing_rx.try_recv().is_ok(){}
   thread::sleep(Duration::from_secs(reconnect_delay));
   reconnect_delay=(reconnect_delay*2).min(60);
  }
 });
 Connection{events,outgoing:outgoing_tx,ready}
}
