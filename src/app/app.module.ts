import { NgModule } from "@angular/core";
import { BrowserModule } from "@angular/platform-browser";

import { AppComponent } from "./app.component";
import { TodoListComponent } from "./todolist/todolist.component";
import { TauriService } from "./tauri.service";
import { FormsModule } from "@angular/forms";
import { CardModule } from "primeng/card";
import { ButtonModule } from "primeng/button";
import { RippleModule } from "primeng/ripple";
import { ListboxModule } from "primeng/listbox";
import { TreeTableModule } from "primeng/treetable";
import { CheckboxModule } from "primeng/checkbox";
import { SpeedDialModule } from "primeng/speeddial";
import { TableModule } from "primeng/table";
import { ToastModule } from "primeng/toast";
import { ScrollTopModule } from "primeng/scrolltop";
import { ConfirmDialogModule } from "primeng/confirmdialog";
import { MessageModule } from "primeng/message";
import { MessagesModule } from "primeng/messages";
import { ConfirmationService, MessageService } from "primeng/api";
import { DialogModule } from "primeng/dialog";
import { BrowserAnimationsModule } from "@angular/platform-browser/animations";
import { InputTextModule } from "primeng/inputtext";

@NgModule({
    declarations: [AppComponent, TodoListComponent],
    imports: [
        BrowserAnimationsModule,
        BrowserModule,
        FormsModule,
        CardModule,
        ButtonModule,
        RippleModule,
        ListboxModule,
        TreeTableModule,
        CheckboxModule,
        SpeedDialModule,
        TableModule,
        ToastModule,
        ScrollTopModule,
        ConfirmDialogModule,
        MessagesModule,
        MessageModule,
        DialogModule,
        InputTextModule,
    ],
    providers: [TauriService, ConfirmationService, MessageService],
    bootstrap: [AppComponent],
})
export class AppModule {
}
