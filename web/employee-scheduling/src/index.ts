import 'instant.page';

import 'bootstrap/dist/css/bootstrap.min.css';
import './index.css';
import './footer.css';
import { contentLoaded, loaded } from 'document-promises';

import importPromiseEmployeeScheduling from './employee_scheduling';

(() => {
    let employeeScheduling;
    const contentLoadedPromise: Promise<void> = contentLoaded.then(async () => {
        employeeScheduling = await importPromiseEmployeeScheduling();
    });
    Promise.all([loaded, contentLoadedPromise]).then(() => {
        console.log('loaded');
        const result = employeeScheduling.solve();
        console.log(result);
    });
})();
