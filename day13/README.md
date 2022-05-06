# Day 13 - Elf Work Clock
Today we are building an elf work clock. This component will pay the elves working at the present factory depending on how much hours they worked.

## How to test
1. Reset your environment: `resim reset`
1. Create the default account: `resim new-account`

### Setup Time Oracle
1. `cd time_oracle`
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate a TimeOracle component: `resim call-function [package_address] UTCTimeOracle new 1`. Take note of the returned ResourceDef somewhere. This is the admin's badge
1. Set the current time to 2021-12-09 12:00:00. `resim run ../set_current_time_1.rtm`

### Setup Work Clock
1. `cd ../elf_work_clock`
1. Build and deploy the blueprint on the ledger: `resim publish .`
1. Instantiate the clock component with 1 worker and 15$/hour pay: `resim call-function [work_package_address] WorkClock new 1 15 [oracle_component_address]`. Take note of the first ResourceDef. This is the elf's badge.
1. Start to work: `resim run ../start_work.rtm`. This will give you a timesheet badge that will allow you to withdraw your money at the end of your shift.
1. Increase the time of the oracle by 8 hours: `resim run ../set_current_time_2.rtm`
1. End your shift: `resim call-method [work_component_address] end_work 1,[timesheet_badge_address]`
1. Look at the resources in your account: `resim show [account_address]`. You should see 120 "Elf Salary" tokens !