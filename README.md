# Hi, this is André!  

Thank you for the opportunity! As a polkadot/substrate developer, I have been getting more into vanilla Rust, so I appreciate the chance to practice.

I am using the latest version of `cargo` and the Rust stable toolchain.  

For this project, I prioritized **readability** and **maintainability** over outright performance. However, I structured the code to ensure scalability, making future optimizations straightforward and easy to integrate. I would love to discuss these optimizations and possible enhancements further. 

I have implemented all the requested features to the best of my ability. I used the serde and csv crates for this project, and also pruned whitespaces and decimal precision is 4 digits.

I have also generated 3 example .csv files that I used to test the application. I tested and calculated the results manually, relying on Rust's type safety to ensure the code was clean and functional. One of the tests is provided by the exercise itself. 
I added another, slightly more complex one that also handles whitespaces. For the last one, I designed a larger example with a lot of moving parts. 
These samples are in the /exercises directory. They can be run with the "cargo run -- examples/hard.csv > accounts.csv" command.

I avoided using the unsafe macro. Error handling was done proportional to the proposed task, since the exercise claims that the data is sanitized in most cases. For the remaining ones, they are caught and thrown. Most errors that occur are also purposeful, so the csv file can be properly smoke-tested.

I optimized the system for a mix of performance and memory resources. The code can be further optimized, but I prefer to maintain a neutral approach until the benchmarks are complete and the bottlenecks are identified.



### Some aspects required interpretation:  

1. **Negative Balances**  
   - I decided to allow negative balances. This mirrors real-world systems, such as Uber, where accounts can temporarily go negative under specific circumstances. For example, if a dispute is filed against an already withdrawn account. 

2. **Account Creation**  
   - I chose to create accounts only upon the first deposit, not on withdrawal or other operations. This ensures accounts are initialized with a positive balance, adhering to a stricter interpretation of account management.  
   
3. **Account Locking**  
   - I decided to treat account locks as a u16 instead of a boolean. This way, multiple locks can be made against one account for different disputes, without fear of a resolve action causing an actively reported account to be unfrozen, for example.

I hope you find the code easy to follow and aligned with the goals of the project. Feedback is always welcome!  

Thank you once again for the opportunity and for reading this far!