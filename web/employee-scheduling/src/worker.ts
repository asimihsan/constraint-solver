import importPromiseEmployeeScheduling from './employee_scheduling';

self.addEventListener('message', handleMessage);

async function handleMessage(e) {
    const employeeScheduling = await importPromiseEmployeeScheduling();
    const result = employeeScheduling.solve(e.data);
    console.log('worker finished.');
    self.postMessage({ result: result });
}
