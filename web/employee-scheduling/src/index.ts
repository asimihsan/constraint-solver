import 'bootstrap/dist/css/bootstrap.min.css';
import './index.css';
import './footer.css';

import 'instant.page';
import {createApp, unref} from 'vue/dist/vue.esm-browser.prod';
import {parsed, contentLoaded} from 'document-promises';
import { DateTime, Duration } from "luxon";

import Worker from "worker-loader!./worker";

import importPromiseEmployeeScheduling from './employee_scheduling';

class Employee {
    readonly id: number;

    constructor(id: number) {
        this.id = id;
    }
}

(() => {
    const now = DateTime.now();
    const NINETY_DAYS = Duration.fromObject({ days: 90 });
    const worker = new Worker('worker');
    const parsedPromise: Promise<void> = parsed.then(async () => {
        await importPromiseEmployeeScheduling();
    });
    Promise.all([parsedPromise, contentLoaded]).then(() => {
        console.log('loaded');
        const app = createApp({
            data() {
                return {
                    startDate: now.toFormat('yyyy-MM-dd'),
                    endDate: (now.plus(NINETY_DAYS)).toFormat('yyyy-MM-dd'),
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
                    id: 7,
                }
            },
            methods: {
                addEmployee() {
                    this.employees.push(new Employee(this.id++));
                },
                removeEmployee(employee) {
                    this.employees = this.employees.filter((x) => x.id != employee.id);
                },
                startSolving() {
                    console.log("start solving");
                    worker.onmessage = (e) => {
                        this.isSolvingButtonActive = true;
                        console.log(e.data.result);
                    };
                    this.isSolvingButtonActive = false;
                    const message = {
                        startDate: this.startDate,
                        endDate: this.endDate,
                        employees: JSON.parse(JSON.stringify(this.employees)),
                        employeeHolidays: [],
                    };
                    console.log(message);
                    worker.postMessage(message);
                },
            }
        })
        app.mount('#app');
    });
})();
