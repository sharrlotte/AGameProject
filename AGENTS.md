# IMPORTANT

- YOU ARE NOT ALLOW TO DECIDE ANYTHING, EVEYTHING MUST BE APPROVED BY ME BEFORE IMPL,
- IF WE MAKE ANY CHANGES, MAKE SURE IT IS RECORDED INSIDE /docs

# PROJECT STRUCTURE

- Look at docs/07-technical-architecture-save-and-roadmap.md for detail
- Structure the project base on feature to create plugin as a small module to enhance "seperation of concern" and speed up compilation

# CODE RULE

- NO unwrap() or unsafe code inside production code
- Do not use tempary solution, hack to sastify the compiler, instead using best practice to make it work
- Make sure bevy system order correct and chained, scoped properly and only run when needed
- Avoid hardcode value, prefer loading data from .ron with async load/hot reload
- Prefer event for communication
- Make sure code is optimized for perfermance
- Dev only code should not exists in Production build
- Respect SOLID principle, when a file > 500 lines then extract

# Files

- Assets folder: game\core\assets
