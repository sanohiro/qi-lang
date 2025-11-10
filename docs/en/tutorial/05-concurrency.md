# Chapter 5: Easy Concurrency and Parallelism

**Time Required**: 35 minutes

Concurrency and parallelism are among Qi's most powerful features. You can achieve **easy, safe, and fast parallel processing**.

---

## Why Concurrency and Parallelism?

### Sequential Processing (Slow)

```qi
(def urls ["https://api1.com" "https://api2.com" "https://api3.com"])

; Process sequentially (takes 3 seconds total)
(map http/get urls)
```

### Parallel Processing (Fast)

```qi
; Process in parallel (completes in about 1 second!)
(urls ||> http/get)
```

---

## Parallel Pipeline: `||>`

`||>` processes each element of a collection **in parallel**.

### Basic Usage

```qi
qi> ([1 2 3 4 5]
     ||> (fn [x] (* x x)))
; => [1 4 9 16 25]
```

**Internally, `pmap` (parallel map) is automatically used.**

### Practical Example 1: Fetching Multiple URLs

```qi
(def urls
  ["https://api.example.com/users/1"
   "https://api.example.com/users/2"
   "https://api.example.com/users/3"])

; Fetch in parallel
(urls
 ||> http/get
 ||> (fn [resp] (get resp :body))
 ||> json/parse)
```

---

## Parallel map: `pmap`

`pmap` is the parallel version of map.

```qi
qi> (pmap (fn [x] (* x 2)) [1 2 3 4 5])
; => [2 4 6 8 10]

qi> (defn heavy-process [x]
      (do
        (sleep 1000)  ; Wait 1 second
        (* x x)))

; Sequential processing (takes 5 seconds)
qi> (map heavy-process [1 2 3 4 5])
; => [1 4 9 16 25]

; Parallel processing (completes in about 1 second)
qi> (pmap heavy-process [1 2 3 4 5])
; => [1 4 9 16 25]
```

---

## Parallel filter: `go/pfilter`

`go/pfilter` is the parallel version of filter.

```qi
qi> (go/pfilter even? [1 2 3 4 5 6 7 8 9 10])
; => [2 4 6 8 10]

(defn is-prime? [n]
  ; Prime number test (heavy processing)
  (if (<= n 1)
    false
    (loop [i 2]
      (if (>= (* i i) n)
        true
        (if (= (% n i) 0)
          false
          (recur (inc i)))))))

; Extract primes in parallel
qi> (go/pfilter is-prime? (range 1 100))
; => [2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71 73 79 83 89 97]
```

---

## Goroutine-Style Concurrency

Qi supports Go-language-style concurrency.

### Creating Channels

```qi
qi> (def ch (go/chan))
```

### Send/Receive

```qi
; Send
qi> (go/send! ch 42)

; Receive
qi> (go/recv! ch)
; => 42
```

### Execute in Goroutine

```qi
; Run in background
qi> (go/run (println "Hello from goroutine!"))
; => "Hello from goroutine!"
```

---

## Practical Example 1: Parallel Downloads

```qi
(defn download-file [url]
  (println f"Downloading: {url}")
  (let [resp (http/get url)]
    (get resp :body)))

(def urls
  ["https://example.com/file1.txt"
   "https://example.com/file2.txt"
   "https://example.com/file3.txt"])

; Parallel download
(def results (urls ||> download-file))

; Or use pmap
(def results (pmap download-file urls))
```

---

## Practical Example 2: Parallel Data Processing

```qi
(defn process-chunk [chunk]
  ; Heavy processing
  (chunk
   |> (filter even?)
   |> (map (fn [x] (* x x)))
   |> (reduce + 0)))

; Split large data
(def data (range 1 1000))
(def chunks
  [(take 250 data)
   (take 250 (drop 250 data))
   (take 250 (drop 500 data))
   (take 250 (drop 750 data))])

; Process each chunk in parallel
(def results (chunks ||> process-chunk))

; Aggregate results
(reduce + 0 results)
```

---

## Practical Example 3: Goroutines and Channels

```qi
(def ch (go/chan))

; Producer
(go/run
  (do
    (go/send! ch 1)
    (go/send! ch 2)
    (go/send! ch 3)
    (go/close! ch)))

; Consumer
(loop [acc []]
  (let [val (go/try-recv! ch)]
    (if (nil? val)
      acc
      (recur (conj acc val)))))
; => [1 2 3]
```

---

## Practical Example 4: Worker Pool

```qi
(defn worker [id ch]
  (loop []
    (let [task (go/try-recv! ch)]
      (if (nil? task)
        (println f"Worker {id} done")
        (do
          (println f"Worker {id} processing: {task}")
          ; Process
          (sleep 100)
          (recur))))))

(def ch (go/chan))

; Start workers
(go/run (worker 1 ch))
(go/run (worker 2 ch))
(go/run (worker 3 ch))

; Submit tasks
(go/send! ch "Task 1")
(go/send! ch "Task 2")
(go/send! ch "Task 3")
(go/send! ch "Task 4")
(go/send! ch "Task 5")

; Close channel
(go/close! ch)
```

---

## Atom: Thread-Safe State Management

With Atoms, you can safely update state from multiple goroutines.

### Basic Usage

```qi
qi> (def counter (atom 0))

; Get value
qi> (deref counter)
; => 0

qi> @counter  ; Short form
; => 0

; Update value
qi> (swap! counter inc)
; => 1

qi> @counter
; => 1
```

### Practical Example: Parallel Counter

```qi
(def counter (atom 0))

; Safely increment from multiple goroutines
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))
(go/run (swap! counter inc))

(sleep 100)  ; Wait for completion

qi> @counter
; => 5
```

---

## Parallel Processing Best Practices

### 1. When to Parallelize?

**Should parallelize when**:
- CPU-intensive processing (computation, transformation)
- I/O-heavy processing (HTTP, file reading)
- Large number of elements (rule of thumb: 100+ elements)

**Should NOT parallelize when**:
- Lightweight processing (per-element processing under 1ms)
- Small number of elements (rule of thumb: under 10 elements)
- Order-dependent processing

### 2. Performance Comparison

```qi
; Lightweight processing - map is faster
(map inc [1 2 3 4 5])  ; Fast

; Heavy processing - pmap is faster
(pmap heavy-process [1 2 3 ... 100])  ; Fast
```

### 3. Debugging Tips

Parallel processing is hard to debug, so verify with sequential processing first:

```qi
; ✅ Verify with sequential processing first
(data |> (map process) |> verify)

; ✅ Parallelize after verification
(data ||> process |> verify)
```

---

## Practice Problems

### Problem 1: Parallel Square Calculation

Write a function that squares each element of a list and calculates the sum in parallel.

```qi
(defn parallel-sum-squares [numbers]
  ; Fill this in
  )

; Test
(parallel-sum-squares [1 2 3 4 5])  ; => 55
```

<details>
<summary>Solution</summary>

```qi
(defn parallel-sum-squares [numbers]
  (numbers
   ||> (fn [x] (* x x))
   |> (reduce + 0)))

; Or use pmap
(defn parallel-sum-squares [numbers]
  (pmap (fn [x] (* x x)) numbers)
  |> (reduce + 0))
```

</details>

### Problem 2: Parallel Filter and Transform

Write a function that extracts even numbers in parallel and doubles them.

```qi
(defn parallel-process [numbers]
  ; Fill this in
  )

; Test
(parallel-process [1 2 3 4 5 6])  ; => [4 8 12]
```

<details>
<summary>Solution</summary>

```qi
(defn parallel-process [numbers]
  (numbers
   |> (go/pfilter even?)
   ||> (fn [x] (* x 2))))
```

</details>

### Problem 3: Access Count with Atom

Write a function that safely updates an access count from multiple goroutines using Atoms.

```qi
(def access-count (atom 0))

(defn record-access []
  ; Fill this in
  )

; Test
(go/run (record-access))
(go/run (record-access))
(go/run (record-access))

(sleep 100)
@access-count  ; => 3
```

<details>
<summary>Solution</summary>

```qi
(def access-count (atom 0))

(defn record-access []
  (swap! access-count inc))
```

</details>

---

## Summary

What you learned in this chapter:

- ✅ Parallel pipeline (`||>`)
- ✅ Parallel map/filter/reduce (`pmap`, `go/pfilter`, `go/preduce`)
- ✅ Goroutine-style concurrency (`go/run`, `go/chan`)
- ✅ Atom (thread-safe state management)
- ✅ Parallel processing best practices

---

## Next Steps

Now that you've mastered concurrency and parallelism, let's learn about **Web Applications and APIs** as the final chapter!

➡️ [Chapter 6: Web Applications and APIs](06-web-api.md)

Learn how to build HTTP servers and JSON APIs.
