/*export interface Type<T = any> extends Function {
    new (...args: any[]): T;
}
export declare function PartialType<T>(classRef: Type<T>): Type<Partial<T>>;
export declare function OmitType<T, K extends keyof T>(classRef: Type<T>, keys: readonly K[]): Type<Omit<T, typeof keys[number]>>;*/

export class Todo {
    title: string;
    description: string;
    subtasks: SubTask[];
    completed: boolean;
    id: number;

    constructor(title: string = "", description: string = "", subtasks: SubTask[] = [], completed: boolean = false, id: number = -1) {
        this.title = title
        this.description = description
        this.subtasks = subtasks
        this.completed = completed
        this.id = id
    }
}

/*export class TodoDto implements Omit<Todo, "subtasks" | "id"> {
    subtasks: SubTask[];
    completed: boolean;
    description: string;
    title: string;
    constructor(title: string, description: string, subtasks: SubTask[], completed: boolean = false) {
        this.title = title
        this.description = description
        this.subtasks = subtasks
        this.completed = completed
    }
}*/

export class SubTask {
    description: string;
    completed: boolean;
    id: number;

    constructor(description: string, completed: boolean, id: number = -1) {
        this.description = description
        this.completed = completed
        this.id = id
    }
}
