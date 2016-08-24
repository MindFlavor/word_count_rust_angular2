# WordCloud using TypeScript, Rust, AngularJS 2, and timdream's WordCloud 

Word clouds are a very attractive way of visually representing the relative importance of words. At a glance you can guess how important a word is relative to the others in the cloud. We will build two components:
1. A text parser that will calculate the relative weight of each word in a given text (we'll use some classical texts as source). The text parser will also expose a JSON REST interface.
2. An AngularJS app that will get the JSON data and then pass it to timdream's WordCloud component in a suitable manner.

The final result is like this:

![](00.gif)

You can find the complete source code in [TO CHANGE](github). We will discuss here the data management parts only.
Our logic is pretty simple: count each occurrence of a word and use it to determine how big the word will appear in the cloud.

## AngularJs 2 App

There are many open source JavaScript libraries that render word clouds, for this post we'll use timdream's one: [http://timdream.org/wordcloud/](http://timdream.org/wordcloud/). Timdream's library comes complete with Typings which is helpful working with TypeScript. This time I will skip the steps required to startup a TypeScript project in Visual Studio Code. If you need more help please refer to my previous post: [How to render SQL Server acyclic blocking graphs using Visual Studio Code, TypeScript, NodeJS and TreantJS – Part 1](http://bit.ly/bcts1). 

### Service

We will start with the service. The service is responsible to retrieve the data (an HTTP GET) and to map it to the strongly typed class. The class for our word weight is a two-liner:

```typescript
export class WordWeight {
    constructor(public word: string,  public count: number) {}
} 
```

The service itself is:

```typescript
@Injectable()
export class WsWordService implements WordService {
    private wordWSUrl = "http://localhost:3005";

    constructor(private http: Http) { }

    public GetWordsCount(name: string): Promise<WordWeight[]> {
        return this.http.get(this.wordWSUrl + "/" + name)
            .toPromise()
            .then((result) => {
                let wws = result.json().map((item) => new WordWeight(item[0], item[1]));

                return wws;
            });
    }
}
```

Notice the ```@Injectable``` attribute. It's required so Angular knows that our service can be injected in our component. We are also requesting an instance of the ```Http``` class in the constructor (via dependency injection). We can then use ```Http``` member to get the JSON from our service. 

Using ```Promise```s, along with the TypeScript's arrow operator, makes the code simple and easy to follow.  

### Component

The Angular component requests the service as prerequisite. It does it in the class decorator under the name ```providers```: 

```typescript
@Component({
    selector: 'my-app',
    templateUrl: '../html/app.component.html',
    providers: [WsWordService]
})
export class AppComponent implements OnInit {
    ...
```

And requests its injection in the constructor: 

```typescript
constructor(private wordService: WsWordService) { }
```

The WordCloud code is in the ```setText``` method. Here, again, we exploit the ```Promise``` in order to get rid of the async HTTP call:

```typescript
setText(text : string) {
    this.wordService.GetWordsCount(text).then((list) => {

        // Give exponential weight to each word count to emphasize
        // small differences. A better algorithm could find the best power
        // based on the variance. 
        let scale = list.map((ww) => new WordWeight(ww.word, Math.pow(ww.count,2)));

        // find maximum weight in the array.
        let max = scale.map((ww) => ww.count).
            reduce((max, cur) => {
                return Math.max(max, cur);
            }, 0);

        // scale each word to a fixed size (this.maxSize) so the more 
        // important word will be this.maxSize and the others will be smaller
        // accordingly.
        scale = scale.map((ww) => new WordWeight(ww.word, (ww.count / max) * this.maxSize));

        // prepare the array required by WordCloud.
        let outarray = scale.map((ww) => [ww.word, ww.count]);

        // Call WordCloud pointing it to our canvas.
        WordCloud(document.getElementById("my_canvas"), {
            list: outarray,
            gridSize: 1,
            minSize: 0,
        });

    });
}
```

That's it for our presentation layer. If we want to try it we can create a mock service returning static values. For example:

```typescript
@Injectable()
export class MockWordService implements WordService {
    public GetWordsCount(name: string): Promise<WordWeight[]> {
        let we = undefined;
        switch (name) {
            case "boccaccio_decameron.txt":
                we = [["donna", 1684], ["uomo", 780], ...];
                break;
            case "cecco_angiolieri_rime.txt":
                we = [["amor", 49], ["tutto", 39], ...];
                break;
            case ...
                ...
        }
        if (we === undefined) {
            return new Promise((res, rej) => {
                rej("not found");
            });
        }
        else {
           let wws = we.map((item) => new WordWeight(item[0], item[1]));

            return new Promise((res, rej) => {
                res(wws);
            });
        }
    }
}
```

Now just inject the mock service instead of the real one and test the app.

## WordCount service

The WordCount service should simply count the occurrences of every word in a given file. This is basically the textbook example of map/reduce. To spice the things up a bit we also want to:

1. Exclude noise words (based on a dictionary).
2. Consolidate similar words (singular/plural, synonyms, etc...).

We also want to create a multithreaded word count service. If we are CPU-bound this should help to speed up the computation on multithreaded machines. This kind of programs are - this example excluded - hard to code. Fortunately there are new languages built from the ground up to tackle the concurrency problems. One I particularly like is Rust ([https://www.rust-lang.org](https://www.rust-lang.org)). While targeted to systems programming Rust can be successfully used for other tasks too. In our case it forces us to think about *ownership* of shared resources, which is good for multithreaded applications (you can find more about it in this blog post [https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html](https://blog.rust-lang.org/2015/04/10/Fearless-Concurrency.html)).

Let's recap the serial logic:
1. Load a file. For each word:
    1. Check if is a noise word.
    2. Lookup the most relevant synonym (if any). If found use it instead of the original word.
    3. Increase the word's count by one.
2. Present the total per word ordered by occurrence count.

If we were able to apply the serial logic in parallel we could come up with:
1. Load the file. For each word:
    1. Find an available thread and dispatch the word to it.
2. Each thread will, for each word received:
    1. Check if it is a noise word.
    2. Lookup the most relevant synonym (if any). If found use it instead of the original word.
    3. Increase the word's count by one.
3. When all the words are dispatched and processed, present the total per word ordered by occurrences.

### The data race

The parallel algorithm is good in theory but poses some immediate questions: "*How do I dispatch a word to a thread?"* or "*How can a thread know when there are no more words to process?"*. 

The biggest problem, however, lies in the data race between threads. The step *Increase the word's count by one* will be performed by multiple threads in parallel and, unless we mediate the access to the *counter* somehow, problems will arise. This is nasty because, in most languages, your code will compile and run even if you were modifying the same object from multiple threads. The result can be a segfault if you're lucky or, **way** worse, miscalculations. 

Bottom line you're own your own. If you are a superstar coder you will prevent problems and end up with an efficient concurrent algorithm. But if you're a normal person like me, chances are you will end up scratching your head at the pseudo random errors. 

The good news is, with Rust, you **cannot** forget about such things. Rust will force you to prevent data races or your code won't even compile. 

Even the apparent innocuous concurrent access to a shared vector should be safe. With Rust it **must** be safe (thankfully). Let's see how. 

### Concurrent access to a shared resource

Let's try to share an immutable vector across threads. A first approach could be:

```rust
fn work() {
    let v = vec![1, 2, 3, 4];

    for _ in 0..3 {
        thread::spawn(|| {
            println!("{:?}", v);
        });
    }
}
```

Here we declare a vector (```v```) and then we spawn 3 threads that will print the vector contents concurrently. Seems legitimate? Well, it isn't. Rust will tell us why:

```
error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
 --> src/main.rs:7:23
  |
7 |         thread::spawn(|| {
  |                       ^^ may outlive borrowed value `v`
8 |             println!("{:?}", v);
  |                              - `v` is borrowed here
  |
help: to force the closure to take ownership of `v` (and any other referenced variables), use the `move` keyword, as shown:
  |         thread::spawn(move || {
```

Rust tell us that the thread may outlive ```work``` (the spawning function). 
But if the spawning function (```work```) terminates, what happens to the vector ```v```? Since the *ownership* of ```v``` belongs to ```work``` and not the threads, as soon as ```work``` finishes the vector would be deallocated. And the threads would end up keeping a reference to dellocated memory. **Bad**.

### Transferring ownership

Rust's new error system will suggest you to move (transfer) the ownership of the vector to the thread. It makes sense: if ```work``` relinquishes the ownership of ```v``` to the thread it will no longer deallocate ```v```. ```v``` would get deallocated at the end of the thread (as it should).

Let's try it (notice the ```move``` keyword before the closure):

```rust
fn work() {
    let v = vec![1, 2, 3, 4];

    for _ in 0..3 {
        thread::spawn(move || {
            println!("{:?}", v);
        });
    }
}
```

Will this work? No. Rust error is like this one:

```bash
error[E0382]: capture of moved value: `v`
 --> src/main.rs:8:30
  |
7 |         thread::spawn(move || {
  |                       ------- value moved (into closure) here
8 |             println!("{:?}", v);
  |                              ^ value captured here after move
<std macros>:2:27: 2:58 note: in this expansion of format_args!
<std macros>:3:1: 3:54 note: in this expansion of print! (defined in <std macros>)
src/main.rs:8:13: 8:33 note: in this expansion of println! (defined in <std macros>)
  |
  = note: move occurs because `v` has type `std::vec::Vec<i32>`, which does not implement the `Copy` trait
```

The error this time is harder to interpret. The hint is in the note: ```v does not implement `Copy` trait```. Why Rust tries to copy our vector instead of simply transferring ownership? Because we are transferring the ownership of *one* vector to *three* threads. Since it's impossible Rust tries to send *three* separate copies of our vector to our *three* threads. But since by default our structures cannot be copied it doesn't work (luckily because it's not something we wanted to do in the first place). 

Go on and try to remove the ```for loop``` and spawn a *single* thread instead of *three*. In that case the move will work. 

### Reference-counted pointers

Clearly moving the ownership of the vector to our threads cannot work without copying it for each thread. This solution is a waste of memory since we know our vector to be immutable. We should be able to access it concurrently without mutexes of any kind. But without copies we cannot move the vector and without move the thread will outlast the vector. 

So, how do we solve the lifespan problem? 

One answer is to use reference counted pointers. These wrappers will keep count of how many references are present at any time and only deallocate the inner object when nobody else is referencing it. It's roughly what a GC-based language would do. Only here we get deterministic deallocation (and also the circular reference problem).

In our case it means the inner object will be referenced by the ```work``` function and by *each* thread. So there are 4 references (1 for ```work``` + 1 for each thread). When ```work``` terminates the reference count will simply drop to 3. And the vector won't be deallocated (yet).

You can read more about it here: [https://doc.rust-lang.org/std/sync/struct.Arc.html](https://doc.rust-lang.org/std/sync/struct.Arc.html) and here [https://doc.rust-lang.org/book/concurrency.html](https://doc.rust-lang.org/book/concurrency.html).

That's our final, working, code:

```rust
fn work() {
    let v = vec![1, 2, 3, 4];

    let av = Arc::new(v);

    for _ in 0..3 {
        // here the reference count is increased.
        // It happens once for each thread.
        let av = av.clone();

        thread::spawn(move || {
            println!("{:?}", av);
        });
    }
}
```

If you try to call it chances are your program will terminate even before the threads have got a chance to print the vector. You could join the threads or, better, coordinate the execution via ```channels```. 

### Channels

Channels are an elegant way to send messages between threads. Rust's channels are special: they are also able to transfer *ownership* of entities between threads. Let's see how with an example. 

We have this super dumb function that returns a Vector:

```rust
fn super_dumb_fn() -> Vec<u64> {
    let mut v = Vec::new();
    for i in 0..100000000 {
        if i % 10000 == 0 {
            v.push(i);
        }
    }

    v
}
```

If we were to run it in a separate thread we could call the ```join``` method of the thread and retrieve the result:

```rust
fn super_dumb_thread() -> Vec<u64> {
    let t = thread::spawn(|| super_dumb_fn());
    t.join().unwrap()
}
```

It works (unwrap notwithstanding) but if we wanted to return data from the thread as soon as it's created? The answer is simple: open a channel between threads.

```rust
fn super_dumb_thread2() -> Vec<u64> {
    let (sx, rx) = channel();

    // here the thread steals the ownership
    // of the send channel. That means it will
    // drop the channel as soon as it finishes,
    // signaling that no more data will be sent.
    thread::spawn(move || {
        for i in 0..100000000 {
            if i % 10000 == 0 {
                sx.send(i).unwrap();
            }
        }

        // here sx will be dropped
    });

    let mut v = Vec::new();
    while let Ok(i) = rx.recv() {
        v.push(i);
    }

    v
}
```

In this example we are sending unsigned longs but we could send anything. In our WordCount service we will be sending in lines to be processed and we will receive back the partial word count. In other words, we will stream lines into the thread to be processed as fast as possibile. When there are no more rows to process we will close the channel. This way the processing thread will know that there is no more data and should produce the results. The results are sent back via another channel which in fact moves the ownership back to the main thread. At this point the processing thread can terminate.

## Putting all together

Now that we are able to share safely an immutable structure between threads and to send ownership using channels we can implement our concurrent word counter method:

```rust
fn process_file(noise_words: Arc<Vec<String>>,
                separators: Arc<Vec<char>>,
                collapser: Arc<Collapser>,
                file_name: &Path)
                -> Result<WordCounter, ProcessFileError> {
    const THREADS: usize = 8;

    let mut send_row_channels = Vec::new();
    let mut relinquish_wc_channels = Vec::new();

    for _ in 0..THREADS {
        let separators = separators.clone();
        let noise_words = noise_words.clone();
        let collapser = collapser.clone();

        let (tx_row, rx_row) = channel::<String>();
        send_row_channels.push(tx_row);

        let (tx_relinquish_wc, rx_relinquish_wc) = channel();
        relinquish_wc_channels.push(rx_relinquish_wc);

        thread::spawn(move || {
            let mut wc = WordCounter::new(separators, noise_words, collapser);
            while let Ok(ref row) = rx_row.recv() {
                wc.process_line(row);
            }

            match tx_relinquish_wc.send(wc) {
                Ok(_) => Ok(()),
                Err(e) => Err(ProcessFileError::RelinquishError(e)),
            }
        });
    }

    let file = File::open(file_name)?;
    let mut file = BufReader::new(file);

    let s = &mut String::new();
    let mut i: usize = 0;

    while file.read_line(s)? > 0 {
        send_row_channels[i % THREADS].send(s.to_string())?;
        i += 1;
        s.clear();
    }

    // close send row channels
    // to do this all we have to do
    // is to drop them.
    for sc in send_row_channels {
        drop(sc);
    }

    // relinquish completed WordCounters and sum (final reduce)
    let mut wc_final = WordCounter::new(separators, noise_words, collapser);

    for ref rx in relinquish_wc_channels {
        let wc = rx.recv()?;

        for (key, val) in wc.hm {
            wc_final.register_word(key, val);
        }
    }

    Ok(wc_final)
}
```

Now all we have to do is to expose this method through a REST interface. For this I've used [Iron framework](https://github.com/iron/iron). I will not show the code here as it's very simple.  

This example of parallel map => parallel reduce => serial reduce is surprisingly fast. I'm sure that better performance can be achieved further optimizing the code but that's beside the point for this example.

---

Happy coding,

[Francesco Cogno](mailto:francesco.cogno@outlook.com)