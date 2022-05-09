import importPromiseEmployeeScheduling from './employee_scheduling';

self.addEventListener('message', handleMessage);

async function handleMessage(e) {
    console.log('worker importing...');
    const employeeScheduling = await importPromiseEmployeeScheduling();
    console.log('worker start...');
    const result = employeeScheduling.solve();
    console.log('worker finished.');
    self.postMessage({ ...e.data, result: result });
}
