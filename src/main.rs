mod configuration;

use std::{ffi::OsStr, sync::Arc};

use mlua::{FromLua, IntoLua, Lua, Result};
use uuid::Uuid;

const SERIAL_TASK_LUA: &'static str = include_str!("../builtin_plugins/serial_task.lua");

struct InternalHandle {
    id: String,
    child: std::process::Child,
}

impl InternalHandle {
    fn expose(&self) -> Handle {
        Handle {
            id: self.id.clone(),
        }
    }
}

struct Handle {
    id: String,
}

impl<'lua> FromLua<'lua> for Handle {
    fn from_lua(value: mlua::Value<'lua>, lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        let table = mlua::Table::from_lua(value, lua)?;
        let id = table.get("id")?;
        Ok(Handle { id })
    }
}

impl<'lua> IntoLua<'lua> for Handle {
    fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value<'lua>> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        Ok(mlua::Value::Table(table))
    }
}

fn main() -> Result<()> {
    let runtime = Lua::new();
    let mut handles: Vec<InternalHandle> = Vec::new();

    let plugin_dir = configuration::get_plugin_directory();

    let _ = std::fs::create_dir_all(&plugin_dir);

    let plugins: Vec<std::path::PathBuf> = std::fs::read_dir(plugin_dir)
        .expect("Failed to read plugin directory")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.exists() && p.is_file())
        .filter(|p| p.extension() == Some(OsStr::new("lua")))
        .collect();

    println!("{:?}", plugins);
    runtime.scope(|scope| {
        let globals = runtime.globals();

        let exec = runtime.create_function(|_, (cmd, args): (String, Vec<String>)| {
            match std::process::Command::new(cmd).args(args).spawn() {
                Ok(mut child) => {
                    let _ = child.wait();
                    Ok(())
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        })?;

        let spawn = scope.create_function_mut(|_, (cmd, args): (String, Vec<String>)| {
            match std::process::Command::new(cmd).args(args).spawn() {
                Ok(child) => {
                    let id = Uuid::now_v7().to_string();
                    let handle = InternalHandle { id, child };
                    let lua_handle = handle.expose();
                    handles.push(handle);
                    Ok(lua_handle)
                }
                Err(e) => Err(mlua::Error::ExternalError(Arc::new(e))),
            }
        })?;

        globals.set("exec", exec)?;
        globals.set("spawn", spawn)?;

        Ok(())
    })?;

    Ok(())
}
