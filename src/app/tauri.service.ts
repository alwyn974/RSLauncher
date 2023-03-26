import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri';
import { SubTask, Todo } from "./todolist/model";

@Injectable({
    providedIn: 'root'
})
export class TauriService {

    constructor() {
    }

    async getTasks(): Promise<Todo[]> {
        let tasks = await invoke<any[]>("get_tasks");
        let todos = [];
        for (let [id, task] of (tasks as unknown as Map<number, ITodo>).entries()) {
            let subtasks = [];
            if (!task.subtasks)
                for (let [key, subtask] of (task.subtasks as Map<number, Subtask>).entries())
                    subtasks.push(new SubTask(subtask.description, subtask.completed, key));
            todos.push(new Todo(task.title, task.description, subtasks, task.completed, id));
        }
        return todos;
    }

    /**
     * Adds a task to the database and returns the id of the task
     * @param task the task to add
     */
    async addTask(task: Todo): Promise<number> {
        return invoke<number>("add_task", {title: task.title, description: task.description, completed: task.completed});
    }

    async deleteTask(taskId: number): Promise<boolean> {
        return invoke<boolean>("delete_task", {id: taskId});
    }

    async completeTask(todo: Todo): Promise<boolean> {
        return invoke<boolean>("edit_task", {id: todo.id, title: todo.title, description: todo.description, completed: todo.completed});
    }

    async editTask(todo: Todo): Promise<boolean> {
        return invoke<boolean>("edit_task", {id: todo.id, title: todo.title, description: todo.description, completed: todo.completed});
    }

    async completeSubTask(taskId: number, subTaskId: number): Promise<boolean> {
        return invoke<boolean>("complete_subtask", {id: taskId, subId: subTaskId});
    }
}

interface Subtask {
    description: string;
    completed: boolean;
}

interface ITodo {
    title: string;
    description: string;
    subtasks: Subtask[];
    completed: boolean;
}