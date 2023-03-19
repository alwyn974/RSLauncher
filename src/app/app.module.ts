import { NgModule } from "@angular/core";
import { BrowserModule } from "@angular/platform-browser";

import { AppComponent } from "./app.component";
import {TodoListComponent} from "./todolist/todolist.component";
import {TauriService} from "./tauri.service";
import {FormsModule} from "@angular/forms";

@NgModule({
  declarations: [AppComponent, TodoListComponent],
  imports: [BrowserModule, FormsModule],
  providers: [TauriService],
  bootstrap: [AppComponent],
})
export class AppModule {}
