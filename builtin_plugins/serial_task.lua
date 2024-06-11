register_task_type("serial_task", {}, function(data)
	for i in pairs(data.subtasks) do
		local task_name = data.subtasks[i]
		local task = tasks[task_name]
		if task == nil then
			print("task", task_name, "does not exist") -- change to error or something
			return
		end
		exec(task["command"], task["args"])
	end
end)
