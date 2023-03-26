import { Component, OnInit } from '@angular/core';
import { TauriService } from '../tauri.service';
import { SubTask, Todo } from "./model";
import { ConfirmationService, MenuItem, MessageService } from "primeng/api";

@Component({
    selector: 'app-todo-list',
    templateUrl: './todolist.component.html',
    styleUrls: ['./todolist.component.css']
})
export class TodoListComponent implements OnInit {
    JSON: JSON = JSON;
    tasks: Todo[] = [];
    addDialogOpened: boolean = false;
    editDialogOpened: boolean = false;
    newTask: Todo = new Todo();
    /*subtasks: SubTask[] = [];*/

    constructor(private tauriService: TauriService, private confirmationService: ConfirmationService, private messageService: MessageService) {
    }

    ngOnInit(): void {
        this.refreshTasks();
    }

    addTask() {
        this.addDialogOpened = false
        if (!this.newTask.title || this.newTask.title.trim().length === 0)
            return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Title is required', key: 'toast'});
        if (!this.newTask.description || this.newTask.description.trim().length === 0)
            return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Description is required', key: 'toast'});

        this.tauriService.addTask(this.newTask).then(taskId => {
            let task: Todo = { ...this.newTask };
            task.id = taskId;
            this.tasks.push(task);

            this.newTask.id = -1;
            this.newTask.title = "";
            this.newTask.description = "";
        });
    }

    deleteTask(taskId: number) {
        this.tauriService.deleteTask(taskId).then((deleted) => {
            if (!deleted)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to delete task', key: 'toast'});
            this.tasks = this.tasks.filter(t => t.id !== taskId);
        });
    }

    editTask(taskId: number) {
        this.editDialogOpened = true;
        this.newTask = this.tasks.find(t => t.id === taskId)!!;
    }

    saveTask() {
        this.editDialogOpened = false;
        this.tauriService.editTask(this.newTask).then((edited) => {
            if (!edited)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to edit task', key: 'toast'});
            this.newTask.id = -1;
            this.newTask.title = "";
            this.newTask.description = "";
            this.refreshTasks();
        });
    }

    deleteSubTask(taskId: number, subTaskId: number) {

    }

    editSubTask(taskId: number, subTaskId: number) {

    }

    refreshTasks() {
        this.tauriService.getTasks().then(tasks => {
            this.tasks = tasks;
        });
    }

    async toggleComplete(task: Todo) {
        const oldCompleted: boolean = task.completed;
        task.completed = !task.completed;
        this.tauriService.completeTask(task).then((completed) => {
            if (!completed){
                task.completed = oldCompleted;
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to complete task', key: 'toast'});
            }
        });
    }
}