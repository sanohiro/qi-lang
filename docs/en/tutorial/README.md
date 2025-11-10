# Qi Tutorial

**Learn Qi from Zero - Hands-On with Practical Code Examples**

This tutorial is structured to help you learn Qi's key features step by step. Each chapter is independent, so you can start with the parts that interest you most.

---

## 📚 Tutorial List

### 1. [Getting Started](01-getting-started.md) ⭐ Must Read
- Installation and environment setup
- Running your first program
- Using the REPL
- Basic data types and operations

**Time Required**: 15 minutes

---

### 2. [Thinking in Pipelines](02-pipelines.md) ⭐ The Essence of Qi
- Pipeline operator (`|>`)
- Designing data flow
- Function composition and chaining
- Practical examples: Text processing, data transformation

**Time Required**: 20 minutes

---

### 3. [Mastering Pattern Matching](03-pattern-matching.md)
- `match` expression basics
- Destructuring data structures
- Guard conditions
- Practical examples: Command processing, message handling

**Time Required**: 25 minutes

---

### 4. [Handling Errors Gracefully](04-error-handling.md) ⭐ Important
- Railway Pipeline (`|>?`)
- `try` and `error?` predicate
- Error propagation design
- Practical examples: File processing, API calls

**Time Required**: 30 minutes

---

### 5. [Easy Concurrency and Parallelism](05-concurrency.md) ⭐ Qi's Strength
- Parallel pipeline (`||>`)
- Goroutine-style concurrency
- Channel communication
- Practical examples: Parallel downloads, data processing

**Time Required**: 35 minutes

---

### 6. [Web Applications and APIs](06-web-api.md)
- Building HTTP servers
- JSON APIs
- Routing and middleware
- Practical examples: RESTful API, CRUD operations

**Time Required**: 40 minutes

---

## 🎯 Learning Paths

### For Beginners (1 hour)
1. Getting Started (15 min)
2. Thinking in Pipelines (20 min)
3. Mastering Pattern Matching (25 min)

### For Practitioners (2 hours)
In addition to the above:
4. Handling Errors Gracefully (30 min)
5. Easy Concurrency and Parallelism (35 min)

### For Web Developers (3 hours)
All tutorials

---

## 💡 Learning Tips

1. **Try in the REPL**: All code examples can be tested in the REPL
   ```bash
   qi  # Launch REPL
   ```

2. **Modify the code**: Don't just copy-paste examples; experiment by changing numbers and data

3. **Don't fear errors**: Error messages are displayed in Japanese (`QI_LANG=ja`)

4. **Start small**: Don't try to write complex code from the start; begin with small functions

---

## 📖 Additional Resources

- **[Quick Reference](../spec/QUICK-REFERENCE.md)** - All features on one page
- **[Complete Language Specification](../spec/)** - Detailed specification
- **[Function Index](../../spec/FUNCTION-INDEX.md)** - Reference for 200+ functions
- **[Style Guide](../style-guide.md)** - Coding conventions

---

## 🚀 Next Steps

After completing the tutorials:

1. **Browse the examples/ directory** for practical code
2. **Start your own project**
3. **Join the community** (GitHub Issues, Discussions)

---

Let's begin with [Getting Started](01-getting-started.md)!
