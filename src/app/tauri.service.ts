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
        let tmpTasks = (await invoke<any>("get_tasks"))
        let tasks = new Map<number, ITodo>();

        for (let [key, value] of Object.entries(tmpTasks)) tasks.set(Number(key), value as ITodo);

        let todos: Todo[] = [];
        for (let [id, task] of tasks.entries()) {
            let subtasks: SubTask[] = [];
            if (Object.keys(task.subtasks).length !== 0) {
                for (let [key, subtask] of Object.entries(task.subtasks))
                    subtasks.push(new SubTask(subtask.description, subtask.completed, Number(key)));
            }
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

    async addSubTask(taskId: number, subtask: SubTask) {
        console.log(taskId, subtask);
        return invoke<number>("add_subtask", {todoId: taskId, description: subtask.description, completed: subtask.completed});
    }

    async deleteSubTask(taskId: number, subtaskId: number) {
        return invoke<boolean>("delete_subtask", {todoId: taskId, id: subtaskId});
    }

    async completeSubTask(taskId: number, subtask: SubTask): Promise<boolean> {
        return invoke<boolean>("edit_subtask", {todoId: taskId, id: subtask.id, description: subtask.description, completed: subtask.completed});
    }

    async editSubTask(taskId: number, subtask: SubTask) : Promise<boolean> {
        return invoke<boolean>("edit_subtask", {todoId: taskId, id: subtask.id, description: subtask.description, completed: subtask.completed});
    }
}

interface Subtask {
    description: string;
    completed: boolean;
}

interface ITodo {
    title: string;
    description: string;
    subtasks: Map<number, Subtask>;
    completed: boolean;
}