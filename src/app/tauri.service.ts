import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/tauri';

@Injectable({
    providedIn: 'root'
})
export class TauriService {

    constructor() { }

    async addTask(task: string): Promise<string> {
        return invoke('add_task', { task });
    }

    async getTasks(): Promise<string[]> {
        return invoke('get_tasks', {});
    }

    async deleteTask(task: string): Promise<void> {
        return invoke('delete_task', { task });
    }
}