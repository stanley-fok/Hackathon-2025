
const planSelectedElement = document.getElementById("planSelected");
const buttons = document.querySelectorAll(".plansButton");
let titleData = ["Plan1","Plan2","Plan3","Plan4"]; 
let descriptionData = ["text1","text2","text3","text4"];   // Data about descriptions should be placed here
let durationData = [1,2,3,4];  // Data about time/duration should be placed here
let interestData = [1,2,3,4];  // Data about interest should be placed here
buttons.forEach((button, index) => {
    let title = titleData[index];
    let description = descriptionData[index]; //Change this once data is implemented unless you import the data into the same array
    let duration = durationData[index];
    let interest = interestData[index];
    const titleElement = document.getElementById(`title${index + 1}`);
    const descriptionElement = document.getElementById(`description${index + 1}`);
    const durationElement = document.getElementById(`duration${index + 1}`);
    const interestElement = document.getElementById(`interest${index + 1}`);
    titleElement.textContent = `${title}`;
    descriptionElement.textContent = `${description}`;
    durationElement.textContent = `${duration} months`;
    interestElement.textContent = `${interest}%`;
})


buttons.forEach((button, index) => {
    button.addEventListener("click", () => {
        buttons.forEach((btn) => {
            btn.value = "Select";
        });
        button.value = "Selected"
        chosenTitle = titleData[index];
        console.log("Plan which is chosen: " + chosenTitle);
        planSelectedElement.textContent = `${chosenTitle}`;
        console.log(`Section ${index + 1} is selected.`);

        
    });
});

