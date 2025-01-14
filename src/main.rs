use std::{error::Error, io, process, env, collections::HashMap};
use csv::Trim;





//* Structs *//

    #[derive(Debug, serde::Deserialize)]
    struct Transaction {
        #[serde(rename = "tx")] // Redundant, since it's already the key of the dictionary. Could be upgraded for performance.
        tx_id: u32,
        #[serde(rename = "type")] // Due to Rust naming conventions, this field cannot be called "type".
        tx_type: String,
        #[serde(rename = "client")]
        client_id: u16,
        #[serde(rename = "amount")]
        amount: Option<f64>, // Option since some transaction types don't have values for "amount"
        #[serde(default)]
        is_disputed: bool,
    }


    #[derive(Debug)]
    struct ClientData {
        available: f64,
        held: f64,
        total: f64,
        total_locks: u16, // There can be more than one simultaneous lock
    }





//* Logic *//

    // Parses a file path as an argument, then uses it to read the corresponding CSV file.
    // After the transaction data is parsed, a map containing the client's data is then returned.
    fn read_csv() -> Result<HashMap<u16, ClientData>, Box<dyn Error>> {

        let args: Vec<String> = env::args().collect();

        if args.len() != 2 {
            return Err("Error! Incorrect call, the usage is 'cargo run -- <file-path> > <output-destination>'".into());
        }

        let file_path = &args[1];

        let mut reader = csv::ReaderBuilder::new().trim(Trim::All).from_path(file_path).unwrap(); // remove whitespaces

        let mut transactions_map : HashMap<u32, Transaction> = HashMap::new();
        let mut client_data_map : HashMap<u16, ClientData> = HashMap::new(); // the return target


        for row in reader.deserialize() {
               
            let transaction: Transaction = row?;

            match transaction.tx_type.as_str() {

                "deposit" => {
                    if let Err(_err) = try_deposit(&mut transactions_map, &mut client_data_map, transaction) {
                        // println!("{}", err);
                        continue;
                    }
                },

                "withdrawal" => {
                    if let Err(_err) = try_withdrawal(&mut transactions_map, &mut client_data_map, transaction) {
                        // println!("{}", err);
                        continue;
                    }
                },

                "dispute" => {
                    if let Err(_err) = try_dispute(&mut transactions_map, &mut client_data_map, transaction) {
                        // println!("{}", err);
                        continue;
                    }
                },

                "resolve" => {
                    if let Err(_err) = try_resolve(&mut transactions_map, &mut client_data_map, transaction) {
                        // println!("{}", err);
                        continue;
                    }
                },

                "chargeback" => {
                    if let Err(_err) = try_chargeback(&mut transactions_map, &mut client_data_map, transaction) {
                        // println!("{}", err);
                        continue;
                    }
                },

                _ => {
                    // println!("Error! Transaction {}'s type is invalid! Ignoring.", transaction.tx_id);
                    continue;
                },
            };
        }

        // println!("Transactions list: {:#?}\n", transactions_map);
        // println!("Client Data: {:#?}\n", client_data_map);

        Ok(client_data_map)
    }


    // Receives the client data map as an input, then sanitizes the data before exporting to the target path. 
    fn write_csv(
        client_data : HashMap<u16, ClientData>,
    ) -> Result<(), Box<dyn Error>> {

        let mut writer = csv::Writer::from_writer(io::stdout());

        // create the header
        writer.write_record(&["client", "available", "held", "total", "locked"])?;

        for (client_id, client) in client_data {
            
            // This formats an f64 with 4 digits of precision
            let formatted_available = (client.available * 10_000.0).round() / 10_000.0; 
            let formatted_held = (client.held * 10_000.0).round() / 10_000.0;
            let formatted_total = (client.total * 10_000.0).round() / 10_000.0;

            let formatted_locked = if client.total_locks > 0u16 {true} else {false};

            writer.write_record(&[
                client_id.to_string(),
                formatted_available.to_string(),
                formatted_held.to_string(),
                formatted_total.to_string(),
                formatted_locked.to_string(),
            ])?;
        }

        writer.flush()?;
        Ok(())
    }





//* Auxiliary Functions *//

    // Tries to deposit funds into an account.
    // A new account is created if none exist with the given ID.
    // This is currently the only way to create a new user entry.
    fn try_deposit(
        transactions_map : &mut HashMap<u32, Transaction>,
        client_data_map : &mut HashMap<u16, ClientData>,
        transaction : Transaction,
    ) -> Result<(), Box<dyn Error>> {

        if transactions_map.contains_key(&transaction.tx_id) {   
            return Err("Error! Transaction ID already exists. Ignoring.".into());  
        }

        let amount = transaction.amount.unwrap();
        if amount <= 0.0f64 {
            return Err("Error! Attempting to deposit a zero or negative balance. Ignoring.".into());
        }    

        let client_data = client_data_map.get_mut(&transaction.client_id);

        if let Some(cd) = client_data {
            if cd.total_locks > 0u16 {
                return Err("Error! Attempting to deposit into a locked account. Ignoring.".into());  
            }

            cd.available += amount;
            cd.total += amount;
        }

        else {
            let cd = ClientData { 
                available: amount, 
                held: 0.0f64, 
                total: amount, 
                total_locks: 0u16,
            }; 

            client_data_map.insert(transaction.client_id, cd);
        }

        transactions_map.insert(transaction.tx_id, transaction);

        Ok(())
    }


    // Tries to withdraw funds from an account.
    // If no matching accounts exist, the transaction is ignored.
    fn try_withdrawal(
        transactions_map : &mut HashMap<u32, Transaction>,
        client_data_map : &mut HashMap<u16, ClientData>,
        transaction : Transaction,
    ) -> Result<(), Box<dyn Error>> {

        if transactions_map.contains_key(&transaction.tx_id) {   
            return Err("Error! Transaction ID already exists. Ignoring.".into());  
        }

        let amount = transaction.amount.unwrap();
        if amount <= 0.0f64 { 
            return Err("Error! Attempting to withdraw a zero or negative balance. Ignoring.".into());   
        }

        let client_data = client_data_map.get_mut(&transaction.client_id);

        if let Some(cd) = client_data {
            if cd.total_locks > 0u16 {
                return Err("Error! Attempting to withdraw from a locked account. Ignoring.".into());  
            }
            if cd.available < 0.0 { // in case a dispute was filed against an already withdrawn balance
                return Err("Error! Attempting to withdraw with negative balance. Ignoring.".into());   
            }

            if cd.available >= amount {
                cd.available -= amount;
                cd.total -= amount;
            }
            else {
                return Err("Error! Attempting to withdraw with insufficient balance. Ignoring.".into()); 
            }
        }

        else {
            return Err("Error! Attempting to withdraw from nonexistent account. Ignoring.".into()); 
        }

        transactions_map.insert(transaction.tx_id, transaction);

        Ok(())
    }



    // I am allowing disputes against both deposits and withdrawals.
    // This should allow the available balance to be negative, since a withdrawal may occur before its dispute.
    // I'm assuming that the client ID for a dispute must match the client's ID in the disputed transaction.
    // Each time a client is flagged with a dispute, they gain 1u16 "total_locks" increment, which freezes their account.
    fn try_dispute(
        transactions_map : &mut HashMap<u32, Transaction>,
        client_data_map : &mut HashMap<u16, ClientData>,
        transaction : Transaction,
    ) -> Result<(), Box<dyn Error>> {

        let transaction_entry = transactions_map.get_mut(&transaction.tx_id);

        if let Some(te) = transaction_entry {
            
            if te.is_disputed {
                return Err("Error! Transaction {} is already disputed! Ignoring.".into()); 
            }
            else if te.client_id != transaction.client_id {
                return Err("Error! Transaction {} is being disputed by an unrelated user! Ignoring.\n".into()); 
            }

            let client_data = client_data_map.get_mut(&transaction.client_id);

            if let Some(cd) = client_data {
                let amount = te.amount.unwrap();
                te.is_disputed = true;
                cd.available -= amount;
                cd.held += amount;
                cd.total_locks = cd.total_locks.checked_add(1u16).unwrap_or(u16::MAX); // prevent overflow
            }

            else {
                return Err("Error! There was no client associated with transaction {}! Ignoring.".into()); 
            }
        }

        else {
            return Err("Error! There is no transaction {} to dispute! Ignoring.".into()); 
        }


        Ok(())
    }


    // I am assuming that the client ID for a resolve must match the client's ID in the disputed transaction.
    // Each time a client's dispute is resolved, they lose 1u16 "total_locks" increment.
    // Their account is only unfrozen if they have 0 "total_locks" increments.
    fn try_resolve(
        transactions_map : &mut HashMap<u32, Transaction>,
        client_data_map : &mut HashMap<u16, ClientData>,
        transaction : Transaction,
    ) -> Result<(), Box<dyn Error>> {



        let transaction_entry = transactions_map.get_mut(&transaction.tx_id);

        if let Some(te) = transaction_entry {
            
            if !te.is_disputed {
                return Err("Error! Transaction {} is not disputed! Ignoring.".into()); 
            }
            else if te.client_id != transaction.client_id {
                return Err("Error! Transaction {} is being disputed by an unrelated user! Ignoring.".into()); 
            }

            let client_data = client_data_map.get_mut(&transaction.client_id);

            if let Some(cd) = client_data {
                let amount = te.amount.unwrap();
                te.is_disputed = false;
                cd.available += amount;
                cd.held -= amount;
                cd.total_locks = cd.total_locks.checked_sub(1u16).unwrap_or(0u16);
            }

            else {
                return Err("Error! There was no client associated with transaction {}! Ignoring.".into()); 
            }
        }

        else {
            return Err("Error! There is no transaction {} to dispute! Ignoring.".into()); 
        }


        Ok(())
    }



    // I am assuming that the client ID for a chargeback must match the client's ID in the disputed transaction.
    // Each time a client's dispute is charged back, they can no longer lose that 1u16 "total_locks" increment.
    // This means their account is permanently frozen (we could assume they would need to contact the service provider).
    fn try_chargeback(
        transactions_map : &mut HashMap<u32, Transaction>,
        client_data_map : &mut HashMap<u16, ClientData>,
        transaction : Transaction,
    ) -> Result<(), Box<dyn Error>> {

        let transaction_entry = transactions_map.get_mut(&transaction.tx_id);

        if let Some(te) = transaction_entry {
            
            if !te.is_disputed {
                return Err("Error! Transaction {} is not disputed! Ignoring.".into()); 
            }
            else if te.client_id != transaction.client_id {
                return Err("Error! Transaction {} is being disputed by an unrelated user! Ignoring.".into()); 
            }

            let client_data = client_data_map.get_mut(&transaction.client_id);

            if let Some(cd) = client_data {
                let amount = te.amount.unwrap();
                cd.held -= amount;
                cd.total -= amount;
            }

            else {
                return Err("Error! There was no client associated with transaction {}! Ignoring.".into()); 
            }
        }

        else {
            return Err("Error! There is no transaction {} to dispute! Ignoring.".into()); 
        }

        Ok(())
    }





//* Main *//

    fn main() {

        let client_data = match read_csv() {
            Ok(cd) => cd,
            Err(e) => {
                println!("{}", e);
                process::exit(1);
            }
        };

        if let Err(e) = write_csv(client_data) {
            println!("Error Writing CSV: {}", e);
            process::exit(1);
        }
        
    }