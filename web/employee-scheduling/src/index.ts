import 'bootstrap/dist/css/bootstrap.min.css';
import './index.css';
import './footer.css';

import 'instant.page';
import { createApp } from 'vue/dist/vue.esm-browser.prod';
import { parsed, contentLoaded } from 'document-promises';

import Worker from "worker-loader!./worker";

import importPromiseEmployeeScheduling from './employee_scheduling';

class Employee {
    readonly id: number;
    holidays: string[];

    constructor(id: number) {
        this.id = id;
        this.holidays = [];
    }
}

(() => {
    const parsedPromise: Promise<void> = parsed.then(async () => {
        await importPromiseEmployeeScheduling();
    });
    Promise.all([parsedPromise, contentLoaded]).then(() => {
        console.log('loaded');
        createApp({
            data() {
                return {
                    employees: [
                        new Employee(0),
                        new Employee(1),
                        new Employee(2),
                        new Employee(3),
                        new Employee(4),
                        new Employee(5),
                        new Employee(6),
                    ],
                    isSolvingButtonActive: true,
                }
            },
            methods: {
                startSolving() {
                    console.log("start solving");
                    const worker = new Worker('worker');
                    worker.onmessage = (e) => {
                        this.isSolvingButtonActive = true;
                        console.log(e.data.result);
                        worker.terminate();
                    };
                    this.isSolvingButtonActive = false;
                    worker.postMessage({ test: 42 });
                },
            }
        }).mount('#app');
    });
})();
