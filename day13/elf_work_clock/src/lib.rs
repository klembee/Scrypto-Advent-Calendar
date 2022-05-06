use scrypto::prelude::*;

import! {
    r#"
    {
      "package_address": "01fe7519e1044c7b182fbc073743df3679d7af10e9eba79bccada2",
      "blueprint_name": "UTCTimeOracle",
      "functions": [
        {
          "name": "new",
          "inputs": [
            {
              "type": "U32"
            }
          ],
          "output": {
            "type": "Tuple",
            "elements": [
              {
                "type": "Custom",
                "name": "ComponentAddress",
                "generics": []
              },
              {
                "type": "Custom",
                "name": "Bucket",
                "generics": []
              }
            ]
          }
        }
      ],
      "methods": [
        {
          "name": "set_current_time",
          "mutability": "Mutable",
          "inputs": [
            {
              "type": "U16"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U8"
            },
            {
              "type": "U64"
            }
          ],
          "output": {
            "type": "Unit"
          }
        },
        {
          "name": "get_time",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "Tuple",
            "elements": [
              {
                "type": "U16"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U8"
              },
              {
                "type": "U64"
              }
            ]
          }
        }
      ]
    }"#
  }

blueprint! {
    struct WorkClock {
        time_oracle: ComponentAddress,
        elf_badge: ResourceAddress,
        time_sheet_minter: Vault,
        salary_vault: Vault,
        hour_salary: Decimal
    }

    impl WorkClock {
        pub fn new(nb_workers: u32, hour_salary: Decimal, time_oracle_address: ComponentAddress) -> (ComponentAddress, Bucket) {
            let elf_badges = ResourceBuilder::new_fungible()
                                    .divisibility(DIVISIBILITY_NONE)
                                    .metadata("name", "Elf Badge")
                                    .initial_supply(nb_workers);

            // Used to create and burn time sheets
            let time_sheet_minter = ResourceBuilder::new_fungible()
                                        .divisibility(DIVISIBILITY_NONE)
                                        .metadata("name", "TimeSheet minter")
                                        .initial_supply(1);

            // Create the tokens that will be used to pay the elfs
            let salary_tokens = ResourceBuilder::new_fungible()
                                    .metadata("name", "Elf Salary")
                                    .initial_supply(100000000000_u64);

            let component = Self {
                time_oracle: time_oracle_address.into(),
                elf_badge: elf_badges.resource_address(),
                time_sheet_minter: Vault::with_bucket(time_sheet_minter),
                salary_vault: Vault::with_bucket(salary_tokens),
                hour_salary: hour_salary
            }
            .instantiate();

            let access_rules = AccessRules::new()
              .method("start_work", rule!(require(elf_badges.resource_address())))
              .default(rule!(allow_all));

            (component.add_access_check(access_rules).globalize(), elf_badges)
        }

        pub fn start_work(&self) -> Bucket {
            // Get the time. Send empty bucket as fee
            let time_oracle: UTCTimeOracle = self.time_oracle.into();
            let (year, month, day, hour, minute, second, unix_time) = time_oracle.get_time();

            // Create a timesheet token
            ResourceBuilder::new_fungible()
                    .divisibility(DIVISIBILITY_NONE)
                    .metadata("name", format!("TimeSheet {}/{}/{} {}:{}:{}", year, month, day, hour, minute, second))
                    .metadata("date", format!("{}", unix_time))
                    .burnable(rule!(require(self.time_sheet_minter.resource_address())), LOCKED)
                    .initial_supply(1)
        }

        pub fn end_work(&mut self, timesheet: Bucket) -> Bucket {
            assert!(timesheet.amount() > Decimal::zero(), "Missing timesheet");

            let time_oracle: UTCTimeOracle = self.time_oracle.into();

            // Get the current time
            let (_, _, _, _, _, _, unix_time) = time_oracle.get_time();

            // No checks here when unwrapping to keep it simple.
            // Keep in mind that anyone could create their own badge to fake the time
            // they started working. I didn't want to do fix this since NFTs have not been implemented yet
            // and they could easily fix this issue.
            let start_time: u64 = borrow_resource_manager!(timesheet.resource_address()).metadata().get("date").unwrap().parse().unwrap();

            // Burn the timesheet
            self.time_sheet_minter.authorize(|| {
                timesheet.burn();
            });

            let hours_worked = (unix_time - start_time) / 3600;
            // Send the salary
            self.salary_vault.take(self.hour_salary * hours_worked)
        }
    }
}
