# IMPORTANT

- YOU ARE NOT ALLOW TO DECIDE ANYTHING, EVEYTHING MUST BE APPROVED BY ME BEFORE IMPL,
- IF WE MAKE ANY CHANGES, MAKE SURE IT IS RECORDED INSIDE /docs, ONLY WRITE SPEC, NO NOTE
- IF YOU SEE ANY CODE THAT VIOLATING ANY RULES BELLOW AND HAVE NO COMMENT TO PREVENT REFACTOR THEN ASK USER FOR PERMISSION TO REWIRTE
- CARGO COMPILE, TEST, CHECK TOOK A REALLY LONG TIME SO MAKE SURE YOU ONLY RUN WHEN NEEDED AND ONLY CHECK WHAT NEEDED

# PROJECT STRUCTURE

- Structure the project base on feature to create plugin as a small module to enhance "seperation of concern" and speed up compilation
- The folder structure must be prepared for future expansion

# CODE RULE

## Common

- The coded must be designed for future proof in mind
- NO unwrap() or unsafe code inside production code
- Do not use tempary solution, hack to sastify the compiler, instead using best practice to make it work
- Make sure bevy system order correct and chained, scoped properly and only run when needed
- Avoid hardcode value, prefer loading data from .ron with async load/hot reload
- Prefer event for communication
- Make sure code is optimized for perfermance
- Dev only code should not exists in Production build
- Respect SOLID principle
- Use macro to replace duplicated code when a function can not do it or using macro is better than using a function
- Use short and concise for name and file name, prefer using 1 word, use folder to group files and prefer creating folder to group for
  future proof when more files is added
- Do not write comment that explain what the code already does or changes, note from old code version
- Single source of struth is required
- Always import type, component, resource, and system sets with `use` declarations at the top of the file; avoid inline fully qualified
  crate paths (e.g., `&crate::...`) in function signatures, system parameters, or function bodies
- Should should be modular, as small as possible in order for futher extension
- Make sure all inputs are validated, constrained
- Always proceed with tasks until everything are done, only stop when there is a road block
- Current code is not perfect, so rewrite them all when needed if we have a new better solution is allowed
- Merge #[cfg()] into least block for more readable code
- A plugin should be always defined in a mod.rs file, for example scene/editor.rs become -> scene/edior/mod.rs, system.rs, component.rs,
  lib.rs, only function defined in system.rs, enum + struct + trait + component should be defined in component, lib.rs is for function that
  used in other folder/module, if the file over 300 lines you can split it into subfolder if needed, function over 100 lines should also
  split
- Bevy component should use a nounce without any prefix or subfix
- The order of defnification in code should be enum + struct + trait + impl on top, function next, test on the bottom
- Prefer enum to create state machine to avoid invalid state for complex state
- Avoid fail silently, we must fail loud or log depend on which is better for UX

## Bevy

- Prefer Single over Query if you only need 1 and that entity always exists 1
- Prefer DespawnOnExit
- Always use prelude for bevy
- We are using bevy 0.20.0, make sure to make use of it new features and do not use old, deprecated features

## Testing

- Run cargo dev after every intergration to make sure no error, run it in the background while you continue to work on the next task, this
  command will run the game fovever and will not exit so you have to terminate it yourself

# Files

- Assets folder: game\assets

# Available tools

- Blender MCP: Read model data, generate model, convert model, etc
- Bevy MCP: Read realtime data, modify, take screenshot
- Python 3
- cargo dev: Run the game in dev env for testing
- Additional scripts inside /tools
