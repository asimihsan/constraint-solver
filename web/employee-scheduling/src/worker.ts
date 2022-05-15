import importPromiseEmployeeScheduling from './employee_scheduling';

(async () => {
    let solver = null;
    const employeeScheduling = await importPromiseEmployeeScheduling();

    async function handleMessage(e) {
        if (e.data.eventType === "start") {
            if (solver !== null) {
                solver.free();
            }
            solver = employeeScheduling.create_solver(e.data);
        }
        employeeScheduling.execute_solver_round(solver);
        const isFinished = employeeScheduling.is_solver_finished(solver);
        const result = employeeScheduling.get_best_solution(solver);
        const iterationInfo = employeeScheduling.get_iteration_info(solver);
        self.postMessage({
            isFinished: isFinished,
            iterationInfo: iterationInfo,
            result: result,
        });
    }

    console.log("worker loaded");
    self.addEventListener('message', handleMessage);
})();


