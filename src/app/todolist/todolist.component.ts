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
    addSubtaskDialogOpened: boolean = false;
    editSubtaskDialogOpened: boolean = false;
    newTask: Todo = new Todo();
    subtask: SubTask = new SubTask();
    todoId: number = -1;

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

    addSubTask() {
        this.addSubtaskDialogOpened = false;
        if (!this.subtask.description || this.subtask.description.trim().length === 0)
            return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Description is required', key: 'toast'});

        this.tauriService.addSubTask(this.todoId, this.subtask).then((subTaskId) => {
            let subtask = { ...this.subtask };
            subtask.id = subTaskId;
            let task = this.tasks.find(t => t.id === this.todoId);
            if (!task)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to find task', key: 'toast'});
            task.subtasks.push(subtask);
            this.subtask = new SubTask();
        });
    }

    deleteSubTask(taskId: number, subTaskId: number) {
        this.tauriService.deleteSubTask(taskId, subTaskId).then((deleted) => {
            if (!deleted)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to delete subtask: ' + subTaskId, key: 'toast'});
            let todo = this.tasks.find((task) => task.id == taskId);
            if (!todo)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to find todo: ' + taskId + ' to update subtasks', key: 'toast'});
            todo.subtasks = todo.subtasks.filter((subtask) => subtask.id !== subTaskId);
        })
    }

    editSubTask(taskId: number, subTaskId: number) {
        this.editSubtaskDialogOpened = true;
        let todo = this.tasks.find((task) => task.id == taskId);
        if (!todo)
            return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to find todo: ' + taskId, key: 'toast'});
        let subtask = todo.subtasks.find((subtask) => subtask.id === subTaskId);
        if (!subtask)
            return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to find substask: ' + subTaskId, key: 'toast'});
        this.subtask = subtask!!;
    }

    saveSubTask() {
        this.editSubtaskDialogOpened = false;
        console.log(this.todoId, this.subtask)
        this.tauriService.editSubTask(this.todoId, this.subtask).then((edited) => {
            if (!edited)
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to edit subtask', key: 'toast'});
            this.todoId = -1;
            this.subtask.id = -1;
            this.subtask.description = "";
            this.refreshTasks();
        });
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

    async toggleSubtaskComplete(task: Todo, subtask: SubTask) {
        const oldCompleted: boolean = subtask.completed;
        subtask.completed = !subtask.completed;
        this.tauriService.completeSubTask(task.id, subtask).then((completed) => {
            if (!completed){
                subtask.completed = oldCompleted;
                return this.messageService.add({severity: 'error', summary: 'Error', detail: 'Failed to complete subtask', key: 'toast'});
            }
        });
    }
}