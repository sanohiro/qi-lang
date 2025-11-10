# Debug Functionality

Qi provides built-in debugging functionality, including tracing, breakpoints, stack traces, and more.

## Overview

The debug module provides the following features:

- **Tracing** - Log function calls and returns
- **Breakpoints** - Pause execution
- **Stack Traces** - Get current call stack
- **Debugger Info** - Check debugger state

## Debug Functions

### debug/trace

Enable/disable trace functionality.

```qi
(debug/trace enabled)
```

**Arguments:**
- `enabled` (boolean) - true=enable, false=disable

**Return Value:** nil

**Example:**

```qi
(defn fibonacci [n]
  (if (<= n 1)
      n
      (+ (fibonacci (- n 1)) (fibonacci (- n 2)))))

(debug/trace true)
(fibonacci 5)
(debug/trace false)
```

Output example:
```
[TRACE] -> fibonacci (test.qi:1)
[TRACE]   -> fibonacci (test.qi:1)
[TRACE]     -> fibonacci (test.qi:1)
[TRACE]     <- fibonacci
[TRACE]     -> fibonacci (test.qi:1)
[TRACE]     <- fibonacci
[TRACE]   <- fibonacci
[TRACE]   -> fibonacci (test.qi:1)
[TRACE]   <- fibonacci
[TRACE] <- fibonacci
```

### debug/break

Set a breakpoint. Execution pauses if a debugger is attached.

```qi
(debug/break)
```

**Arguments:** none

**Return Value:** nil

**Example:**

```qi
(defn process-data [data]
  (println "Processing:" data)
  (debug/break)  ;; Pause here
  (if (> data 100)
      "Large"
      "Small"))

(process-data 150)
```

### debug/stack

Get the current call stack as a string.

```qi
(debug/stack)
```

**Arguments:** none

**Return Value:** string - stack trace

**Example:**

```qi
(defn inner []
  (debug/stack))

(defn middle []
  (inner))

(defn outer []
  (middle))

(println (outer))
```

Output example:
```
  #0 inner at test.qi:2
  #1 middle at test.qi:5
  #2 outer at test.qi:8
```

### debug/info

Get the current state of the debugger.

```qi
(debug/info)
```

**Arguments:** none

**Return Value:** map - debug information

Keys in returned map:
- `:enabled` (boolean) - Whether debugger is enabled
- `:state` (string) - Debugger state ("Running", "Paused", etc.)
- `:stack-depth` (integer) - Current stack depth

**Example:**

```qi
(println (debug/info))
;=> {:enabled false}

;; When debugger is enabled
;=> {:enabled true :state "Running" :stack-depth 0}
```

## Usage Examples

### Stack Trace on Error

```qi
(defn error-handler []
  (println "Error occurred at:")
  (println (debug/stack)))

(defn divide [a b]
  (if (= b 0)
      (error-handler)
      (/ a b)))

(divide 10 0)
```

### Debugging with Trace

```qi
(defn factorial [n]
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

(debug/trace true)
(println "Result:" (factorial 5))
(debug/trace false)
```

### Conditional Breakpoint

```qi
(defn process-batch [items]
  (each (fn [item]
          (when (> item 1000)
            (debug/break))  ;; Stop only for large values
          (println "Processing:" item))
        items))

(process-batch [10 50 1200 5])
```

## VSCode Debug Support

Qi supports VS Code integrated debugger:

- GUI-based breakpoint setting
- Step execution (Step In, Step Over, Step Out)
- Variable inspection
- Watch expression evaluation
- Call stack navigation

## Enabling the Debugger

The debugger can be enabled via startup options:

```bash
# Launch with debug mode
qi --debug script.qi

# Attachable via DAP (listen on specified port)
qi --debug-port 5678 script.qi
```

## Performance

Debug functionality affects performance, so do not use in production environments:

- Enabling `debug/trace` logs every function call, significantly reducing execution speed
- `debug/break` and `debug/stack` are relatively lightweight but should not be called frequently
- `debug/info` is lightweight with minimal performance impact

## Related Topics

- [Test Framework](14-stdlib-test.md) - Testing and Assertions
- [Profiling](ROADMAP.md#profiling) - Performance Measurement
- [Logging](09-modules.md#log) - Structured Logging
