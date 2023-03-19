import { Component, OnInit } from '@angular/core';
import { TauriService } from '../tauri.service';

@Component({
  selector: 'app-todo-list',
  templateUrl: './todolist.component.html',
  styleUrls: ['./todolist.component.css']
})
export class TodoListComponent implements OnInit {
  tasks: string[] = [];
  newTask: string = '';

  constructor(private tauriService: TauriService) { }

  ngOnInit(): void {
    this.tauriService.getTasks().then(tasks => {
      this.tasks = tasks;
    });
  }

  addTask() {
    this.tauriService.addTask(this.newTask).then(task => {
      this.tasks.push(task);
      this.newTask = '';
    });
  }

  deleteTask(task: string) {
    this.tauriService.deleteTask(task).then(() => {
      this.tasks = this.tasks.filter(t => t !== task);
    });
  }
}