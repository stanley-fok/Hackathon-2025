const planSelectedElement = document.getElementById("planSelected");
const planImageElement = document.getElementById("planImage"); //get image element
const buttons = document.querySelectorAll(".plansSection");

let titleData = ["Current Account", "Savings Account", "Stocks & Shares Account"];
let imageData = ["current account.png", "savings account.png", "stocks&shares account.png"];
let descriptionData = ["", "", ""];
let durationData = [1, 2, 3];
let interestData = [1, 2, 3];

buttons.forEach((button, index) => {
  button.addEventListener("click", () => {
    console.log("Clicked");
    buttons.forEach((btn) => {
      btn.value = "Select";
    });
    button.value = "Selected";

    let chosenTitle = titleData[index];
    let chosenImage = imageData[index];

    console.log("Plan which is chosen: " + chosenTitle);
    planSelectedElement.textContent = `${chosenTitle}`;

    planImageElement.src = `plots/${chosenImage}`;
    planImageElement.style.display = "block"; //Choose image and display the chosen one
  });
});
