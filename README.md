scraplirs
=========

scraplirs -- scrap(e c)li (but in rust!) --  is a Rust library focused on connecting to devices, specifically network 
devices (routers/switches/firewalls/etc.) via SSH and NETCONF. (lol we are pretty far from netconf things happening 
for now!)

*This is very much a work in progress, is probably not idiomatic rust, and may never be completed! If you are a 
rustacean and interested in this get in touch!*

For now there is only a basic generic driver and "system" transport implementations -- you can run the simple 
example like `cargo run example --generic_driver` -- you probably will need to tweak the connection settings 
(host/user/etc.) in there though!
